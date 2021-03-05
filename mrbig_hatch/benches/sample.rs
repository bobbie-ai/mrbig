// ############################################################################
// #                                                                          #
// # mrbig_hatch/benches/sample.rs                                            #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Sample benchmark test using Criterion crate.                #
// ############################################################################


//! Hatch server benchmarks
//!
//! This testing module implements a bunch of benchmarks to test the efficiency
//! of `Mr. Big Hatch` server.

// Import external dependencies
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Import local dependencies
use mycrate::fibonacci;

/// Bench the average throughput of the hatch server
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
