// ############################################################################
// #                                                                          #
// # mrbig_cli/src/commands/login.rs                                          #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Log to the remote K8S cluster.                              #
// ############################################################################

//! Login command.
//!
//! This command allows to log on remote Kubernetes development cluster.
//! An alternative to the password is to give a SSL certificate.
//! TLS connection is used to securize the connections to the remote development
//! platform.
//!
//! # Example
//!
//! ```ignore
//!
//! > mrbig login --user [username] --password [encrypted_password]
//! ```

// Import external dependencies
use colored::*;
use structopt::StructOpt;

// Import internal dependencies
use crate::errors::CliError as Error;
use crate::errors::Result;

/// Login command options
///
/// Each option is a parameter that can be passed at command-line, when
/// running the Mr. Big CLI application.
///
/// # Example
/// ```ignore
///
/// > mrbig setup my_project --with-template "simple-grpc-microservice"
/// ```
#[derive(Debug, StructOpt)]
pub struct LoginCommandOptions {
    /// Name of the user
    #[structopt(long = "user", short = "u")]
    pub user: Option<String>,

    /// Encrypted password (when passed at command-line)
    #[structopt(long = "password", short = "p")]
    pub password: Option<String>,
}

// Default setup commnand options builder
impl Default for LoginCommandOptions {
    fn default() -> Self {
        Self {
            user: None,
            password: None,
        }
    }
}

/// Executes the login command, passing arguments given at command-line
pub fn execute(options: LoginCommandOptions) -> Result<()> {
    debug!("Execute login command with options: {:?}", options);
    Ok(())
}
