// ############################################################################
// #                                                                          #
// # mrbig_cli/src/settings.rs                                                #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Application's configuration.                                #
// ############################################################################


//! Application settings.


use config;

pub struct Settings {}

// Internal dependencies
use utilities::errors::{Result};


// Public (API) functions
impl Settings {

  /// Loads application's configuration file
  pub fn load() -> Option<&Config> {

    let mut settings = config::Config::default();

    settings
      // Add in `./settings.toml`
      .merge(config::File::with_name("settings")).unwrap()
      // Add in settings from the environment (with a prefix of APP)
      // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
      .merge(config::Environment::with_prefix("APP")).unwrap();
  }
}
