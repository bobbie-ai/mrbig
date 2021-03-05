// ############################################################################
// #                                                                          #
// # mrbig_cli/src/macros.rs                                                  #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Macros definitionn.                                         #
// ############################################################################

//! Bunch of useful macros


/// Display debugging message
#[cfg(debug)]
macro_rules! debug {
    ($x:expr) => { dbg!($x) }
}

#[cfg(not(debug))]
#[allow(unused_macros)]
macro_rules! debug {
    ($x:expr) => { std::convert::identity($x) }
}
