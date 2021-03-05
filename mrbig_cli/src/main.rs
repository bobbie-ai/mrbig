// ############################################################################
// #                                                                          #
// # mrbig_cli/src/main.rs                                                    #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Application's entry point.                                  #
// ############################################################################

// Temporarily allow unused
#![allow(dead_code, unused)]
// ensure Rust 2018 idiom is used
#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

// Import macros
#[macro_use]
extern crate anyhow;

// Import external modules
use std::env;
use std::panic;
use structopt::StructOpt;

// Import local modules
mod utilities;

//use utilities::ApplicationInstaller;
use mrbig_cli::{commands::Command, errors::Result, Cli, SHELL};

// Starts the application folks !!!
pub fn main() {
    // setup logging facility
    env_logger::init();

    // catch panic state
    catch_panic_state();

    if let Err(error) = run() {
        eprintln!("Error: {}", error);

        ::std::process::exit(1);
    }
}

// Starts the application.
fn run() -> Result<()> {
    // parse command-line flags first (to track requested log level)
    let matches = Cli::from_args();

    SHELL.set_log_level(matches.log_level);

    // execute the requested command
    Command::execute(matches.command)
}

// Install the hook to trap panic state.
//
// This function displays a human-readable message on the console when
// the application panics before exiting.
fn catch_panic_state() {
    let meta = human_panic::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let default_hook = panic::take_hook();

    if env::var("RUST_BACKTRACE").is_err() {
        panic::set_hook(Box::new(move |info: &panic::PanicInfo<'_>| {
            // first call the default hook that prints to standard error
            default_hook(info);

            // then display a more readable message using `human_panic` tool
            let file_path = human_panic::handle_dump(&meta, info);

            human_panic::print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }));
    }
}
