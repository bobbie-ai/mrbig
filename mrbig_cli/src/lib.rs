// ############################################################################
// #                                                                          #
// # mrbig_cli/src/lib.rs                                                     #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Application API.                                            #
// ############################################################################

#![deny(missing_docs)]

//! Mr. Big CLI
//!
//! [Mr. Big](https://mrbig.io/) is a framework to build and manage Rust cloud-native
//! applications based on (micro-) services and propelled using Kubernetes.
//! This CLI tool allows to manage various components, including, for instance:
//!
//! - Project scaffolding helpers
//! - Code synchronization process, called `hatch` (as it passed to modifications
//! made in the application's source locally to the remote Kubernetes operating
//! platform)
//! - Dashboard for deployed services monitoring and management, implemented as
//! VSCode extension and as a Web application (using WebAssembly and Rust)
//!
//! # Getting started
//!
//! (Work in progress)

#![allow(
  missing_docs,
  dead_code,
  unused_imports            // TODO: to be activated again (just during dev, this is annoying)
)]

// External dependencies
//extern crate cargo_metadata;
//extern crate curl;
//#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate anyhow;
//extern crate liquid;
//extern crate serde;
//#[macro_use] extern crate serde_derive;
//extern crate serde_ignored;
//extern crate serde_json;
//extern crate which;

// Export local modules
pub mod commands;
pub mod errors;
mod utilities;
mod shell;

// Internal dependencies
use structopt::StructOpt;
use shell::{Shell, LogLevel};
use commands::Command;

/// Export global (and thread-safe) shell instance
pub static SHELL: Shell = Shell::new();


/// Command-line parameters
#[derive(Debug, StructOpt)]
#[structopt(name = "Main command")]
pub struct Cli {

  /// Log verbosity is based off the number of v used (i.e. -v, -vv, -vvv ...)
  #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
  pub verbosity: u8,

  #[structopt(long = "log-level", default_value = "info")]
  /// The maximum level of messages that should be logged. [possible values: info, warn, error]
  pub log_level: LogLevel,

  /// Subcommand to be executed (e.g. `mrbig setup ...`)
  #[structopt(subcommand)]
  pub command: Command
}
