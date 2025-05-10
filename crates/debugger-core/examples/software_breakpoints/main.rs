use std::path::PathBuf;

use debugger_core::*;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let path = PathBuf::from("./example-programs/multiple_prints/multiple_prints");
    let mut debugger = Debugger::new_with_forked_child(path).unwrap();

    // Break at fn_c
    debugger.set_breakpoint_at_text_offset(0x118f);
    // Hits breakpoint the first time, just before printing "C"
    debugger.continue_execution();
    // "C" printed for the first time, hits breakpoint before printing "C" again
    debugger.continue_execution();
    // Continues and exists with code 0
    debugger.continue_execution();
}
