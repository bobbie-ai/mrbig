// ############################################################################
// #                                                                          #
// # mrbig_cli/src/errors.rs                                                  #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Errors definition and processing.                           #
// ############################################################################


//! Error types and routines.
//!
//! This module describes various errors.

use std::io;
use thiserror::Error as ThisError;

// Short hand for standard [`Result`] type.
//
// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Generic command-line errors enumeration
#[derive(ThisError, Debug)]
pub enum CliError {

  #[error("unknown log level name")]
  UnknownLogLevel,

  #[error("cannot log to the remote cluster")]
  LoginToCluster(#[from] io::Error),

  #[error("unknown cli error... ╰U╯☜(◉ɷ◉ )")]
  UnknownCliError,
}

/// Scaffold command errors enumeration
///
/// Errors triggered during project scaffolding processes.
#[derive(ThisError, Debug)]
pub enum ScaffoldError {

  #[error("cannot create project's filesystem structure")]
  ProjectFilesystemStructure(#[from] io::Error),

  #[error("unknown scaffold error... ╰U╯☜(◉ɷ◉ )")]
  UnknownScaffoldError,
}

/// Setup command errors enumeration
///
/// Errors triggered when setting up Mr. Big for a project.
#[derive(ThisError, Debug)]
pub enum SetupError {

  #[error("cannot create configuration file")]
  CannotCreateConfigFile(#[from] io::Error),

  #[error("unknown setup error... ╰U╯☜(◉ɷ◉ )")]
  UnknownSetupError,
}

/// Hatch command errors enumeration
///
/// Errors triggered when trying to manage the hatch syncronizatio server.
#[derive(ThisError, Debug)]
pub enum HatchError {

  #[error("cannot synchronize project's source code")]
  CannotSynchronizeProject(#[from] io::Error),

  #[error("unknown hatch error... ╰U╯☜(◉ɷ◉ )")]
  UnknownHatchError,
}
