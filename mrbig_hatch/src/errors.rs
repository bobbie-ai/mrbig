// ############################################################################
// #                                                                          #
// # mrbig_hatch/src/errors.rs                                                #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Hatch errors definition.                                    #
// ############################################################################


//! Error types and routines.
//!
//! This module describes various errors the hatch server and tools can trigger.

//! Error types and routines.
//!
//! This module describes various errors.


// Import external dependencies
use std::error::Error as StdError;
use std::result::Result as StdResult;
use thiserror::Error as ThisError;

// Short hand for standard [`Result`] type.
//
// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = StdResult<T, Box<dyn StdError>>;


/// Hatch server errors enumeration
#[derive(ThisError, Debug)]
pub enum HatchError {

  #[error("unknown log level name")]
  UnknownLogLevel,

  #[error("unknown cli error... ╰U╯☜(◉ɷ◉ )")]
  UnknownHatchError,
}
