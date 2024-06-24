# G-Tree Experiments

Statistics-gathering and benchmarking for the [G-tree paper](https://github.com/AljoschaMeyer/g_trees).

To gather statistics, execute `cargo run --bin stats`. To benchmark search, run `cargo bench`.

G-trees are implemented in [`src/lib.rs`](./src/lib.rs) and closely follow the pseudocode from the paper. In other words, they are not particularly optimized.

K-lists are implemented in [`src/klist.rs`](./src/klist.rs). Apologies for the code quality.

For testing, we have some pretty exhaustive fuzz-tests in [`fuzz`](./fuzz). See the [rust fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html) for setup details. Run via `cargo fuzz run gtree`, `cargo fuzz run join`, etc.

License: MIT