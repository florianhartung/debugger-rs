use std::{num::ParseIntError, path::PathBuf, process::ExitCode, str::FromStr};

use clap::{Parser, Subcommand};
use clap_repl::{
    ClapEditor,
    reedline::{DefaultPrompt, DefaultPromptSegment},
};
use debugger_core::{ContinueExecutionOutcome, Debugger};
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
    #[clap(alias = "b")]
    Break {
        #[clap(value_parser=clap::value_parser!(BreakpointLocation))]
        /// An offset where the breakpoint will be placed as a decimal (123) or hexadecimal number (0x123). The prefix "text:" can be used to specify an offset relative to the start of the text section.
        location: BreakpointLocation,
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
enum BreakpointLocation {
    Offset(u64),
    TextOffset(u64),
    // Symbol(String),
}

impl FromStr for BreakpointLocation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix("text:").map_or_else(
            || clap_num::maybe_hex::<u64>(s).map(BreakpointLocation::Offset),
            |s| clap_num::maybe_hex::<u64>(s).map(BreakpointLocation::TextOffset),
        )
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
            Ok(ContinueExecutionOutcome::Other) => {}
            Err(err) => {
                println!("Got error while continuing execution: {err}");
                std::process::exit(0);
            }
        },
        ReplCommand::Break { location } => {
            let res = match location {
                BreakpointLocation::Offset(offset) => debugger.set_breakpoint_at(offset),
                BreakpointLocation::TextOffset(offset) => {
                    debugger.set_breakpoint_at_text_offset(offset)
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
                        println!("- {function}");
                    }
                }
                Err(err) => println!("Failed to list all functions: {err}"),
            },
        },
    });

    ExitCode::SUCCESS
}
