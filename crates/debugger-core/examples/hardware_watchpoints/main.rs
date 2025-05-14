use std::path::PathBuf;

use debugger_core::watchpoint::*;
use debugger_core::*;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let path = PathBuf::from("./example-programs/write_to_global_var/write_to_global_var");
    let mut debugger = Debugger::new_with_forked_child(path).unwrap();

    let watchpoint_exec = Watchpoint::Execution;
    let watchpoint_write = Watchpoint::Data {
        condition: WatchpointDataCondition::Write,
        length: WatchpointLength::FourBytes,
    };
    // Break at before_write
    debugger.set_watchpoint_at(0x401136, watchpoint_exec);
    // Break at after_write
    debugger.set_watchpoint_at(0x40115d, watchpoint_exec);
    // Break at write to a
    debugger.set_watchpoint_at(0x404030, watchpoint_write);
    // Hit breakpoint of before_write()
    debugger.continue_execution();
    // Hit watchpoint (write to a)
    debugger.continue_execution();
    // Hit watchpoint again
    debugger.continue_execution();
    // Hit watchpoint yet again?
    debugger.continue_execution();
    // Hit breakpoint of after_write()
    debugger.continue_execution();
    // Continues and exits with code 0
    debugger.continue_execution();
}
