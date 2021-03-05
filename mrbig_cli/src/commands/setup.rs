// ############################################################################
// #                                                                          #
// # mrbig_cli/src/commands/initialize.rs                                     #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Seminal configuration of Mr. Big suppport for a project.    #
// ############################################################################


//! Setup command.
//!
//! This command initializes Mr. Big support for a project.
//! As a result, a hidden folder called `.mrbig` is created at the root of the
//! project's folder, that contains a (TOML) configuration file called
//! `mrbig_config.toml`.
//!
//! # Example
//!
//! In the following example, Mr. Big is initializes for an existing project
//! called `my_project`.
//!
//! ```no_run
//!
//! > cd my_project
//! > mrbig setup
//! ```
//!
//! The setup command can also be used to create a new project using a given
//! template, so that to scaffold a seminal project structure with sample
//! micro-services or serverless components. Here, a new project called `new_project
//! is created using a template called `sample-microservice`. To get a list
//! of all available templates, enter the command `mrbig list templates`. Templates
//! are distributed from a Git repository (no need to give the repository's URL for
//! default templates, but of course of your if you're using a personal template
//! repository).
//!
//! ```no_run
//!
//! > cd ~
//! > mrbig setup new_project --from-template "simple-grpc-microservice"
//! ```
//!
//! # See
//! - Official `Mr. Big` [templates repository](https://github.com/mrbig-templates) on GitHub.

// Import external dependencies
use colored::*;

// Import internal dependencies
use crate::errors::Result;
use crate::errors::SetupError as Error;


/// Setup command options
///
/// Each option is a parameter that can be passed at command-line, when
/// running the Mr. Big CLI application.
///
/// # Example
/// ```no_run
///
/// > mrbig setup my_project --from-template "simple-grpc-microservice"
/// ```
#[derive(Debug, StructOpt)]
pub struct SetupCommandOptions {

  /// The path of the new project
  #[structopt(parse(from_os_str))]
  pub project_path: Option<PathBuf>,

  /// Create a new project based on the given template
  #[structopt(long = "from-template", short = "t")]
  pub from_template: Option<String>,

  // TODO (mab) - Add more options
}

// Default setup commnand options builder
impl Default for SetupCommandOptions {
  fn default() -> Self {
    Self {
      project_path: None,
      with_template: None,
    }
  }
}

/// Executes the setup command, passing arguments given at command-line
pub fn execute(options: SetupCommandOptions) -> Result<()> {
  debug!("Execute setup command with options: {:?}", options);
  Ok(())
}

