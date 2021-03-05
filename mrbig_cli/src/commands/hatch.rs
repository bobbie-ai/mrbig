// ############################################################################
// #                                                                          #
// # mrbig_cli/src/commands/hatch.rs                                          #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: CLI command used to manage the hatch service.               #
// #              The hatch deploys source code modifications on-the-fly to   #
// #              the staging/production cloud platform, hence bluring the    #
// #              frontier between development and operating platforms, while #
// #              reducing time-to-production and squashing development's     #
// #              life-cycle to a single operation (no need to take care      #
// #              about building/deploying Docker, service meshing, and so    #
// #              on and so forth). In other words Mr. Big Hatch allows for   #
// #              non-stop innovation and one-stop cloud deployment.          #
// ############################################################################

//!

// Import external dependencies
use colored::*;

// Import internal dependencies
use crate::errors::Result;
use crate::errors::HatchError as Error;


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
  #[structopt(long = "with-template", short = "t")]
  pub with_template: Option<String>,

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

