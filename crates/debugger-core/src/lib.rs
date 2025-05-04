use std::path::PathBuf;

use log::{debug, error, info};
use nix::{
    sys::{ptrace, signal::Signal, wait::WaitStatus},
    unistd::{ForkResult, Pid},
};

mod libc_wrappers;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to attach debugger to child process")]
    ChildAttachment,
    #[error("failed to continue execution of child process")]
    ContinueExecution,
    #[error("failed get executable path of pid {0}")]
    ReadExecutablePath(Pid),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Debugger {
    executable_path: PathBuf,
    tracee_pid: Pid,
}

pub enum ContinueExecutionOutcome {
    ProcessExited(i32),
    Other,
}

impl Debugger {
    pub fn new_with_forked_child(executable_path: PathBuf) -> Result<Self> {
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

        let debugger = Self {
            executable_path,
            tracee_pid: child_pid,
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

        let proc_exe_path = PathBuf::from(format!("/proc/{}/exe", pid));
        let executable_path = nix::fcntl::readlink(&proc_exe_path)
            .map_err(|_| {
                error!("Could not get executable path from pid {pid}");

                Error::ReadExecutablePath(pid)
            })?
            .into();

        let debugger = Self {
            executable_path,
            tracee_pid: pid,
        };

        info!("Successfully attached debugger to running process with pid {pid}");

        Ok(debugger)
    }

    pub fn continue_execution(&mut self) -> Result<ContinueExecutionOutcome> {
        nix::sys::ptrace::cont(self.tracee_pid, None).map_err(|errno| {
            error!("failed ptrace cont call: {errno}");

            Error::ContinueExecution
        })?;

        let wait_status = nix::sys::wait::waitpid(self.tracee_pid, None).map_err(|errno| {
            error!("failed waitpid after continuing execution: {errno}");

            Error::ContinueExecution
        })?;

        match wait_status {
            WaitStatus::Exited(_pid, exit_code) => {
                Ok(ContinueExecutionOutcome::ProcessExited(exit_code))
            }
            _ => Ok(ContinueExecutionOutcome::Other),
        }
    }
}
