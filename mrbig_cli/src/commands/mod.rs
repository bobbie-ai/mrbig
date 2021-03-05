// ############################################################################
// #                                                                          #
// # mrbig_cli/src/commands/mod.rs                                            #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: CLI command structures and processing routines.             #
// ############################################################################

//! Command structures, parsing and processing.
//!
//! Command types are enumerated in the [Command] enumeration. Each command is
//! translated to a [Clap]() subcommand using macros provided by the [structopt]
//! package.

#![allow(clippy::redundant_closure)]

// Import external dependencies
use log::info;
use std::time::{Duration, Instant};
use structopt::StructOpt;

// Import internal dependencies
mod login;
mod scaffold;

use crate::errors::Result;
use login::LoginCommandOptions;
use scaffold::ScaffoldCommandOptions;

/// Command types enumeration
///
/// Each command is translated into a [Clap subcommand](https://kbknapp.github.io/clap-rs/clap/struct.SubCommand.html)
/// using macros provided by the [structopt traits](https://docs.rs/structopt/0.2.0/structopt/#traits).
/// Let's have some examples on how to use this command.
///
/// # Examples
///
/// Here, **support for Mr. Big is configured for a project** called `my_project`. The `--with-template` option allows to
/// scaffold a new (or existing) project with some seminal Rust files inside, implementing a sample micro-service based
/// on Mr. Big's core Rust macros (providing the boring boilerplate for microservice or serverless components) or a
/// more complex project implementing several micro-services with a service meshing in place, for instance. For more
/// information on available templates, please consult the official [Mr. Big templates repository]() on GitHub.
///
/// ```ignore
/// > mrbig setup my_project --with-template sample-microservice
/// ```
///
/// Now imagine you would like to deeply clean the project, here's the CLI command you can use:
///
/// ```ignore
/// > mrbig clean --all
/// ```
#[derive(Debug, StructOpt)]
pub enum Command {
    /// Initialize Mr. Big for a given project
    #[structopt(name = "setup", alias = "init")]
    Setup,

    /// Scaffold a new project from a given solution template
    #[structopt(name = "scaffold", alias = "generate")]
    Scaffold(ScaffoldCommandOptions),

    /// Cleaning the project's junks
    #[structopt(name = "clean")]
    Clean,

    /// Login into the development sandbox
    #[structopt(name = "login")]
    Login(LoginCommandOptions),
}

// Public functions implementation
impl Command {
    /// Executes the given command.
    pub fn execute(command: Command) -> Result<()> {
        let start_time = Instant::now();

        match command {
            Command::Login(options) => {
                login::execute(options)?;
            }

            Command::Setup => {
                unimplemented!("Setup command is not yet implemented...");
            }

            Command::Scaffold(options) => {
                scaffold::execute(options)?;
            }

            Command::Clean => {
                unimplemented!("Clean command is not yet implemented...");
            }
        }

        // return time elapsed for running the command
        let duration = Command::_format_command_duration(start_time.elapsed());
        info!("Done in {}.", &duration);

        Ok(())
    }
}

// Private functions implementations
impl Command {
    // Formats elapsed time (i.e. duration) so that it looks nice when displayed in a console
    fn _format_command_duration(duration: Duration) -> String {
        let seconds = duration.as_secs();

        if seconds >= 60 {
            format!("{}m {:02}s", seconds / 60, seconds % 60)
        } else {
            format!("{}.{:02}s", seconds, duration.subsec_nanos() / 10_000_000)
        }
    }
}
