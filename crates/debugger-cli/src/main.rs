use std::{convert::Infallible, fmt, path::PathBuf, process::ExitCode, str::FromStr};

use clap::{Parser, Subcommand};
use clap_repl::{
    ClapEditor,
    reedline::{DefaultPrompt, DefaultPromptSegment},
};
use debugger_core::{ContinueExecutionOutcome, Debugger, watchpoint::*};
use envconfig::Envconfig;
use log::LevelFilter;

#[derive(Envconfig)]
struct EnvvarConfig {
    #[envconfig(from = "RUST_LOG", default = "INFO")]
    pub log_level: LevelFilter,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ProgramArgs {
    executable_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(name = "")]
enum ReplCommand {
    #[clap(alias = "c")]
    Continue,
    #[clap(alias = "s")]
    Step {
        #[clap(default_value_t = 1)]
        steps: u32,
    },
    #[clap(alias = "b")]
    Break {
        #[clap(value_parser=clap::value_parser!(BreakpointLocation))]
        /// An address where the breakpoint will be placed as a decimal (123) or hexadecimal number (0x123). The prefix "text:" can be used to specify an offset relative to the start of the text section. Also symbol names can be used.
        location: BreakpointLocation,
        #[clap(value_parser=clap::value_parser!(BreakpointType), default_value_t=BreakpointType::Software)]
        breakpoint_type: BreakpointType,
    },
    #[clap(alias = "w")]
    Watch {
        #[clap(value_parser=clap::value_parser!(BreakpointLocation))]
        /// An address where the breakpoint will be placed as a decimal (123) or hexadecimal number (0x123). The prefix "text:" can be used to specify an offset relative to the start of the text section. Also symbol names can be used.
        location: BreakpointLocation,
        #[clap(value_parser=clap::value_parser!(WatchCondition))]
        condition: WatchCondition,
        length: usize,
    },
    #[clap(alias = "i")]
    Info {
        #[command(subcommand)]
        command: InfoCommand,
    },
    #[clap(alias = "q")]
    Quit,
}

#[derive(Debug, Clone)]
struct WatchCondition(WatchpointDataCondition);

impl FromStr for WatchCondition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "w" | "write" => Ok(WatchCondition(WatchpointDataCondition::Write)),
            "rw" | "read_write" => Ok(WatchCondition(WatchpointDataCondition::ReadWrite)),
            other => Err(format!("Unknown data watchpoint condition {other}")),
        }
    }
}

#[derive(Debug, Clone)]
enum BreakpointType {
    Hardware,
    Software,
}

impl fmt::Display for BreakpointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakpointType::Software => write!(f, "software"),
            BreakpointType::Hardware => write!(f, "hardware"),
        }
    }
}

impl FromStr for BreakpointType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hardware" | "hard" | "h" => Ok(BreakpointType::Hardware),
            "software" | "soft" | "s" => Ok(BreakpointType::Software),
            other => Err("Unknown breakpoint type {other}"),
        }
    }
}

#[derive(Debug, Clone)]
enum BreakpointLocation {
    Address(u64),
    TextOffset(u64),
    Symbol(String),
}

impl FromStr for BreakpointLocation {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(offset) = s
            .strip_prefix("text:")
            .and_then(|s| clap_num::maybe_hex::<u64>(s).ok())
        {
            Ok(BreakpointLocation::TextOffset(offset))
        } else if let Ok(offset) = clap_num::maybe_hex::<u64>(s) {
            Ok(BreakpointLocation::Address(offset))
        } else {
            Ok(BreakpointLocation::Symbol(s.to_owned()))
        }
    }
}

#[derive(Debug, Subcommand)]
enum InfoCommand {
    Functions,
}

fn main() -> std::process::ExitCode {
    // For development/testing only
    let _ = dotenvy::dotenv();

    let config = match EnvvarConfig::init_from_env() {
        Ok(config) => config,
        Err(error) => {
            println!("Got error while parsing environment variables: {error}");
            return ExitCode::FAILURE;
        }
    };
    env_logger::builder().filter_level(config.log_level).init();

    let args = ProgramArgs::parse();

    let mut debugger = match Debugger::new_with_forked_child(args.executable_path) {
        Ok(debugger) => debugger,
        Err(err) => {
            println!("Failed to create debugger instance: {err}");
            return ExitCode::FAILURE;
        }
    };

    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Empty,
        right_prompt: DefaultPromptSegment::Empty,
    };
    let rl = ClapEditor::<ReplCommand>::builder()
        .with_prompt(Box::new(prompt))
        .build();

    rl.repl(|command| match command {
        ReplCommand::Continue => match debugger.continue_execution() {
            Ok(ContinueExecutionOutcome::ProcessExited(code)) => {
                println!("Process exited with code {code}. Quitting...");
                std::process::exit(0);
            }
            Ok(ContinueExecutionOutcome::BreakpointHit(address)) => {
                println!("Hit breakpoint at address {address}");
            }
            Ok(ContinueExecutionOutcome::WatchpointHit((address, watchpoint))) => {
                println!("Hit watchpoint {watchpoint:?} at address 0x{address:012x}");
            }
            Ok(ContinueExecutionOutcome::Other) => {}
            Err(err) => {
                println!("Got error while continuing execution: {err}");
                std::process::exit(0);
            }
        },
        ReplCommand::Step { steps } => match debugger.step_instructions(steps) {
            Ok(new_pc) => {
                println!("Stepped {steps} instructions, pc now at 0x{new_pc:012x}");
            }
            Err(err) => {
                println!("Encountered error while stepping instructions: {err}");
            }
        },
        ReplCommand::Break {
            location,
            breakpoint_type,
        } => {
            let watchpoint = Watchpoint::Execution;

            let res = match location {
                BreakpointLocation::Address(address) => match breakpoint_type {
                    BreakpointType::Software => debugger.set_breakpoint_at(address),
                    BreakpointType::Hardware => debugger.set_watchpoint_at(address, watchpoint),
                },
                BreakpointLocation::TextOffset(offset) => match breakpoint_type {
                    BreakpointType::Software => debugger.set_breakpoint_at_text_offset(offset),
                    BreakpointType::Hardware => {
                        debugger.set_watchpoint_at_text_offset(offset, watchpoint)
                    }
                },
                BreakpointLocation::Symbol(symbol_name) => {
                    match debugger.find_symbol_address_by_name(&symbol_name) {
                        Ok(Some(address)) => match breakpoint_type {
                            BreakpointType::Software => {
                                debugger.set_breakpoint_at_text_offset(address)
                            }
                            BreakpointType::Hardware => {
                                debugger.set_watchpoint_at_text_offset(address, watchpoint)
                            }
                        },
                        Ok(None) => {
                            println!("No symbol found");
                            return;
                        }
                        Err(err) => {
                            println!("Got error during symbol look up: {err}");
                            return;
                        }
                    }
                }
            };
            if let Err(err) = res {
                println!("Failed to set breakpoint: {err}");
            }
        }
        ReplCommand::Watch {
            location,
            condition,
            length,
        } => {
            // Not pretty, should probably have value parser for WatchpointLength
            let watchpoint_length = match WatchpointLength::try_from(length) {
                Ok(length) => length,
                Err(err) => {
                    println!("Got error while parsing watchpoint length: {err}");
                    return;
                }
            };

            let watchpoint = Watchpoint::Data {
                condition: condition.0,
                length: watchpoint_length,
            };

            let res = match location {
                BreakpointLocation::Address(address) => {
                    debugger.set_watchpoint_at(address, watchpoint)
                }
                BreakpointLocation::TextOffset(offset) => {
                    debugger.set_watchpoint_at_text_offset(offset, watchpoint)
                }
                BreakpointLocation::Symbol(symbol_name) => {
                    match debugger.find_symbol_address_by_name(&symbol_name) {
                        Ok(Some(address)) => {
                            debugger.set_watchpoint_at_text_offset(address, watchpoint)
                        }
                        Ok(None) => {
                            println!("No symbol found");
                            return;
                        }
                        Err(err) => {
                            println!("Got error during symbol look up: {err}");
                            return;
                        }
                    }
                }
            };
            if let Err(err) = res {
                println!("Failed to set breakpoint: {err}");
            }
        }
        ReplCommand::Quit => {
            // TODO kill children of debugger
            std::process::exit(0);
        }
        ReplCommand::Info { command } => match command {
            InfoCommand::Functions => match debugger.list_function_symbols() {
                Ok(functions) => {
                    println!("List of all functions:");
                    for function in functions {
                        println!(
                            "- {} ({:#x}) ",
                            function.name.unwrap_or("---"),
                            function.offset,
                        );
                    }
                }
                Err(err) => println!("Failed to list all functions: {err}"),
            },
        },
    });

    ExitCode::SUCCESS
}
