// ############################################################################
// #                                                                          #
// # mrbig_hatch/src/lib.rs                                                   #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Hatch server library.                                       #
// ############################################################################


// Import modules
pub mod errors;
pub mod server;

// Rexport local modules
pub use server::HatchServer;
