use std::{collections::HashMap, path::PathBuf, mem};

use log::{debug, error, info};
use nix::{
    sys::{ptrace, signal::Signal, wait::WaitStatus},
    unistd::{ForkResult, Pid},
    libc,
};

use memory_map::ProcMemoryMaps;
use watchpoint::Watchpoint;

mod libc_wrappers;
mod memory_map;
pub mod symbols;
pub mod watchpoint;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("executable path provided is not readable")]
    NoReadExecutablePath(PathBuf),
    #[error("invalid elf file")]
    InvalidElfFile(#[from] elf::ParseError),
    #[error("failed to attach debugger to child process")]
    ChildAttachment,
    #[error("failed to continue execution of child process")]
    ContinueExecution,
    #[error("failed to read child memory at address 0x{0:8x}")]
    ReadMemory(u64),
    #[error("failed to write child memory at address 0x{0:8x}")]
    WriteMemory(u64),
    #[error("failed to read registers of tracee")]
    ReadRegisters,
    #[error("failed to write registers of tracee")]
    WriteRegisters,
    #[error("A breakpoint at address 0x{0:8x} already exists")]
    BreakpointExists(u64),
    #[error("failed get executable path of pid {0}")]
    ReadExecutablePath(Pid),
    #[error("{0} is not a valid debug register index")] 
    DebugRegisterIndex(usize),
    #[error("{0} ist not a valid value for the length of a watchpoint")]
    WatchpointLengthValue(usize),
    #[error("cannot add more than 4 watchpoints")]
    MaxNumWatchpoints,
    #[error("failed to step one instruction")]
    SingleStep,
    #[error("an io error occured")]
    IoError(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Debugger {
    executable_path: PathBuf,
    tracee_pid: Pid,
    memory_maps: ProcMemoryMaps,
    breakpoints: HashMap<u64, i64>,
    watchpoints: HashMap<usize, (u64, Watchpoint)>,
    executable_data: Vec<u8>,
}

pub enum ContinueExecutionOutcome {
    ProcessExited(i32),
    BreakpointHit(u64),
    WatchpointHit((u64, Watchpoint)),
    Other,
}

impl Debugger {
    pub fn new_with_forked_child(executable_path: PathBuf) -> Result<Self> {
        // read the file data first, even though its needed only later. this validates that the executable file is readable
        let Ok(executable_data) = std::fs::read(&executable_path) else {
            return Err(Error::NoReadExecutablePath(executable_path));
        };

        debug!("Forking process and executing {executable_path:?} in a child process...");

        // SAFETY: Fork is generally safe to call, because it clones the entire process.
        let child_pid = match unsafe { nix::unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => child,
            Ok(ForkResult::Child) => {
                let Err(error) = nix::sys::ptrace::traceme().and_then(|()| {
                    // TODO: Use a better exec (v variant) for passing args
                    libc_wrappers::execl(&executable_path)
                });

                // Log the error for now, in the future we might want to return a proper error to the debugger.
                error!("Failed execution inside new child process {executable_path:?}: {error}");

                std::process::exit(1);
            }
            Err(_) => {
                error!("fork failed");
                return Err(Error::ChildAttachment);
            }
        };

        // Wait for SIGTRAP, which is sent after successfull execl (see execve(2)).
        let wait_status = nix::sys::wait::waitpid(child_pid, None).map_err(|errno| {
            error!("waitpid unexpectedly failed: {errno}");
            Error::ChildAttachment
        })?;
        match wait_status {
            WaitStatus::Stopped(_pid, Signal::SIGTRAP) => {}
            other => {
                error!(
                    "tracee was unexpectedly not stopped by signal SIGTRAP. wait_status={other:?}"
                );
                return Err(Error::ChildAttachment);
            }
        }

        let memory_maps = ProcMemoryMaps::from_pid(child_pid)?;

        let debugger = Self {
            executable_path,
            tracee_pid: child_pid,
            memory_maps,
            breakpoints: HashMap::new(),
            watchpoints: HashMap::new(),
            executable_data,
        };

        info!(
            "Successfully attached debugger to child process with pid {}",
            debugger.tracee_pid
        );

        Ok(debugger)
    }

    pub fn new_with_existing_process(pid: nix::libc::pid_t) -> Result<Self> {
        debug!("Attaching debugger to running process with pid {pid}");

        let pid = Pid::from_raw(pid);

        if let nix::Result::Err(errno) = ptrace::attach(pid) {
            error!("Failed to attach debugger to process ({errno})");

            return Err(Error::ChildAttachment);
        }

        let proc_exe_path = PathBuf::from(format!("/proc/{pid}/exe"));
        let executable_path = nix::fcntl::readlink(&proc_exe_path)
            .map_err(|_| {
                error!("Could not get executable path from pid {pid}");

                Error::ReadExecutablePath(pid)
            })?
            .into();

        let Ok(executable_data) = std::fs::read(&executable_path) else {
            return Err(Error::NoReadExecutablePath(executable_path));
        };

        let memory_maps = ProcMemoryMaps::from_pid(pid)?;

        let debugger = Self {
            executable_path,
            tracee_pid: pid,
            memory_maps,
            breakpoints: HashMap::new(),
            watchpoints: HashMap::new(),
            executable_data,
        };

        info!("Successfully attached debugger to running process with pid {pid}");

        Ok(debugger)
    }

    pub fn get_text_offset_address(&mut self, offset: u64) -> u64 {
        let text_section = self.memory_maps.get_text_section();

        text_section.range_from - text_section.offset + offset
    }

    pub fn set_breakpoint_at(&mut self, breakpoint_address: u64) -> Result<()> {
        let breakpoint_address_ptr = breakpoint_address as *mut core::ffi::c_void;

        let replaced_word =
            ptrace::read(self.tracee_pid, breakpoint_address_ptr).map_err(|errno| {
                error!("Could not read from address 0x{breakpoint_address:8x?}: {errno}");

                Error::ReadMemory(breakpoint_address)
            })?;

        let breakpoint_word = (replaced_word & (!0xFFi64)) | 0xCCi64;
        ptrace::write(self.tracee_pid, breakpoint_address_ptr, breakpoint_word).map_err(
            |errno| {
                error!("Could not write to address 0x{breakpoint_address:8x?}: {errno}");

                Error::WriteMemory(breakpoint_address)
            },
        )?;

        self.breakpoints.insert(breakpoint_address, replaced_word);

        info!("Set breakpoint at {breakpoint_address:08x}");

        Ok(())
    }

    pub fn set_breakpoint_at_text_offset(&mut self, text_offset: u64) -> Result<()> {
        let breakpoint_address = self.get_text_offset_address(text_offset);

        if self.breakpoints.contains_key(&breakpoint_address) {
            return Err(Error::BreakpointExists(breakpoint_address));
        }

        self.set_breakpoint_at(breakpoint_address)
    }

    pub fn set_watchpoint_at(&mut self, address: u64, watchpoint: Watchpoint) -> Result<()> {
        let number_watchpoints = self.watchpoints.len();
        let debug_register = number_watchpoints; 

        if number_watchpoints == 4 {
            return Err(Error::MaxNumWatchpoints);
        }
    
        let mut debug_control = self.get_debug_control()?; 

        self.set_debug_register(debug_register, address as i64);
        debug_control |= 1 << (debug_register * 2);

        match watchpoint {
            Watchpoint::Execution => {},
            Watchpoint::Data { condition, length } => {
                debug_control &= !(0b1111 << (16 + (4 * debug_register)));
                debug_control |= (condition as i64) << (16 + (4 * debug_register));
                debug_control |= (length as i64) << (16 + (4 * debug_register));
            },
        }

        self.set_debug_control(debug_control)?;

        let value_pair = (address, watchpoint);
        self.watchpoints.insert(debug_register, value_pair);
        
        Ok(())
    }

    pub fn set_watchpoint_at_text_offset(&mut self, text_offset: u64, watchpoint: Watchpoint) -> Result<()> {
        let breakpoint_address = self.get_text_offset_address(text_offset);

        self.set_watchpoint_at(breakpoint_address, watchpoint)
    }

    pub fn get_tracee_pc(&self) -> Result<u64> {
        let regs = ptrace::getregs(self.tracee_pid).map_err(|errno| {
            error!("Could not read registers of tracee: {errno}");

            Error::ReadRegisters
        })?;

        Ok(regs.rip)
    }

    pub fn set_tracee_pc(&self, new_pc: u64) -> Result<()> {
        let mut regs = ptrace::getregs(self.tracee_pid).map_err(|errno| {
            error!("Could not read registers of tracee: {errno}");

            Error::ReadRegisters
        })?;
        regs.rip = new_pc;

        ptrace::setregs(self.tracee_pid, regs).map_err(|errno| {
            error!("Could not write registers of tracee: {errno}");

            Error::WriteRegisters
        })?;

        Ok(())
    }

    fn wait_for_tracee(&self) -> Result<WaitStatus> {
        nix::sys::wait::waitpid(self.tracee_pid, None).map_err(|errno| {
            error!("failed waitpid after stepping one instruction: {errno}");

            Error::ContinueExecution
        })
    }

    pub fn step_instructions(&self, steps: u32) -> Result<u64> {
        for _ in 0..steps {
            ptrace::step(self.tracee_pid, None).map_err(|errno| {
                error!("failed to single step execution {errno}");

                Error::SingleStep
            })?;

            self.wait_for_tracee()?;
        }

        self.get_tracee_pc()
    }

    pub fn continue_execution(&mut self) -> Result<ContinueExecutionOutcome> {
        nix::sys::ptrace::cont(self.tracee_pid, None).map_err(|errno| {
            error!("failed ptrace cont call: {errno}");

            Error::ContinueExecution
        })?;

        let wait_status = self.wait_for_tracee()?;

        match wait_status {
            WaitStatus::Exited(_pid, exit_code) => {
                info!("Process exited with code {exit_code}");
                Ok(ContinueExecutionOutcome::ProcessExited(exit_code))
            }
            WaitStatus::Stopped(_pid, Signal::SIGTRAP) => {
                if let Ok(status) = self.get_debug_status() {
                    for i in 0..4 {
                        if status & (1 << i) > 0 {
                            let (address, watchpoint) = self.watchpoints.get(&i).expect("breakpoint to exist");
                            info!("Hit watchpoint {watchpoint:?} at address 0x{address:012x?}");
                            return Ok(ContinueExecutionOutcome::WatchpointHit((*address, *watchpoint)));
                        }
                    }
                }

                let stopped_pc = self.get_tracee_pc()?;
                let breakpoint_pc = stopped_pc - 1;

                // There has to be a better mechanism to detect a software breakpoint
                // Replaces the software breakpoint with the original word at the breakpoint address,
                // steps a single instruction and replaces the int3 instruction back into the breakpoint address
                if let Some(replaced_word) = self.breakpoints.get(&breakpoint_pc) {
                    info!("Hit Software Breakpoint at {breakpoint_pc:08x}");

                    self.set_tracee_pc(breakpoint_pc)?;
                    ptrace::write(
                        self.tracee_pid,
                        breakpoint_pc as *mut core::ffi::c_void,
                        *replaced_word,
                    )
                    .map_err(|errno| {
                        error!("failed to write to address {breakpoint_pc:08x}: {errno}");

                        Error::WriteMemory(breakpoint_pc)
                    })?;

                    ptrace::step(self.tracee_pid, None).map_err(|errno| {
                        error!("failed ptrace step call: {errno}");

                        Error::ContinueExecution
                    })?;

                    self.wait_for_tracee()?;

                    self.set_breakpoint_at(breakpoint_pc)?;

                    Ok(ContinueExecutionOutcome::BreakpointHit(breakpoint_pc))
                } else {
                    Ok(ContinueExecutionOutcome::Other)
                }
            }
            _ => Ok(ContinueExecutionOutcome::Other),
        }
    }
}
