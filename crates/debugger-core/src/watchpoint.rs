use std::convert::TryFrom;

use nix::sys::ptrace;
use log::*;

use crate::{Debugger, Error, Result}; 

pub enum DebugRegisterOffsets {
    B0 = 0,
    B1 = 1,
    B2 = 2,
    B3 = 3,
    DebugStatus = 6,
    DebugControl = 7,
}

#[derive(Debug, Clone, Copy)]
pub enum WatchpointDataCondition {
    Write = 0b01,
    ReadWrite = 0b11,
}

#[derive(Debug, Clone, Copy)]
pub enum WatchpointLength {
    OneByte = 0b00,
    TwoBytes = 0b01,
    FourBytes = 0b11,
    EightBytes = 0b10,
}

impl TryFrom<usize> for WatchpointLength {
    type Error = Error;

    fn try_from(val: usize) -> Result<Self>{
        match val {
            1 => Ok(WatchpointLength::OneByte),
            2 => Ok(WatchpointLength::TwoBytes),
            4 => Ok(WatchpointLength::FourBytes),
            8 => Ok(WatchpointLength::EightBytes),
            _ => Err(Error::WatchpointLengthValue(val)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Watchpoint {
    Execution,
    Data {
        condition: WatchpointDataCondition,
        length: WatchpointLength,
    },
}

impl Debugger {
    fn get_b0_offset(&self) -> usize {
        std::mem::offset_of!(nix::libc::user, u_debugreg)
    }

    pub fn get_debug_register(&self, index: usize) -> Result<i64> {
        let offset = self.get_b0_offset() + (index * std::mem::size_of::<i64>());

        let value = ptrace::read_user(self.tracee_pid, offset as *mut core::ffi::c_void).map_err(|_| Error::ReadRegisters)?;

        Ok(value)
    }

    pub fn set_debug_register(&self, index: usize, data: i64) -> Result<()> {
        let offset = self.get_b0_offset() + (index * std::mem::size_of::<i64>());

        ptrace::write_user(self.tracee_pid, offset as *mut core::ffi::c_void, data).map_err(|errno| {
            error!("Failed to write to debug register {index}: {errno}");

            Error::WriteRegisters
        })?;

        Ok(())
    }

    pub fn get_debug_control(&self) -> Result<i64> {
        self.get_debug_register(DebugRegisterOffsets::DebugControl as usize)
    }

    pub fn set_debug_control(&self, value: i64) -> Result<()> {
        self.set_debug_register(DebugRegisterOffsets::DebugControl as usize, value)
    }

    pub fn get_debug_status(&self) -> Result<i64> {
        self.get_debug_register(DebugRegisterOffsets::DebugStatus as usize)
    }
}