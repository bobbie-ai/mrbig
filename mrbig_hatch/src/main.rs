// ############################################################################
// #                                                                          #
// # mrbig_hatch/src/main.rs                                                  #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Hatch server's entry point.                                 #
// ############################################################################

// Temporarily allow unused
#![allow(dead_code, unused)]

#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

// Import external dependencies
use std::env;
use std::panic;

// Import local dependencies
mod errors;
mod server;

use server::HatchServer;


#[tokio::main]
async fn main() {

  // catch panic and control signals (like Ctrl-C, for instance)
  catch_panic_state();

  let result = HatchServer::new().start().await;

  println!("Result: {:?}", result);

  match result {

    Err(error) => {
      println!("Cannot run hatch server; error is {}", error)
    }

    _ => { println!("Bye folks... thanks for using Mr. Big") }
  }
}

// Install the hooks to trap signals and catch the panic state.
//
// This function setup the OS-related signals, like to trap the signal raised
// when entering `Ctrl-C` keys and to catch the state when the application
// enters in panic mode.
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
