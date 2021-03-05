// ############################################################################
// #                                                                          #
// # mrbig_cli/src/utilities/mod.rs                                           #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Utilities module declaration.                               #
// ############################################################################


//! Utilities module
//!
//! This module contains convenient functions. These functions or traits must
//! not be part of a public API.

pub mod cargo;
pub mod docker;
pub mod emojis;
//pub mod installer;

//pub use installer::ApplicationInstaller;
pub use cargo::CargoManifest;
