# FBAS Reward Distributor

Framework for the computation of a node's influence and reward distribution in
a Federated Byzantine Agreement Systems (FBASs) like
[Stellar](https://www.stellar.org/).

The framework is suitable for usage as both a binary and a library and can mainly do the following:

- read node data in [stellarbeat](https://stellarbeat.io/)'s JSON format;
- rank the nodes using a variety of implemented algorithms;
- based on the rankings, compute a reward distribution for each node.

## Required toolchain

- A working [Rust](https://www.rust-lang.org) environment
    - Install: https://www.rust-lang.org/tools/install

- A minimal gcc and g++ toolchain is required by some of the dependencies.

## Build and optionally run tests

1. Build:
```
cargo build --release
```
2. Tests
```
cargo test --release
```

## Usage as a binary

1. Command line arguments
```
cargo run --release -- {distribute | rank} [-i -p -r reward] <fbas-path> {node-rank|power-index-approx|power-index-enum}

    - fbas-path: Path to file describing the FBAS.
        If no path is passed, the program will attempt to read from the command line.
    - i: Ignore inactive nodes in the FBAS. Optional. Default = false.
    - r reward: reward value that is to be distributed - only used with the rank subcommand. Default = 1.
    - p: Include the nodes' public keys in the output. Default = false.

The rank subcommand is similar to distribute only with the exception that it only calculates the nodes' weights without allocating rewards.
The output is always a sorted list of tuples: (NodeID, Public Key (where available), Ranking, [Reward]).

```
2. Compute how 10 units should be distributed among the nodes in the `mobilecoin_nodes_2021-10-22.json` using a graph-theoretic (Noderank) metric.
```
cargo run --release -- distribute -r 10 test_data/mobilecoin_nodes_2021-10-22.json node-rank
```
or using the Shapley-Shubik power index
```
cargo run --release -- distribute -r 10 test_data/mobilecoin_nodes_2021-10-22.json power-index-enum
```
This algorithm computes the players' Shapley-Shubik indices via enumeration in `O(2^n)` time, and is therefore not recommended for larger FBASs.

As an alternative, we provide an polynomial time approximation implementation using [Castro et al.'s algorithm](https://www.sciencedirect.com/science/article/abs/pii/S0305054808000804) based on sampling. 
3. Rank the nodes using the approximation algorithm and use 1000 samples for the approximation
```
cargo run --release -- rank test_data/mobilecoin_nodes_2021-10-22.json power-index-approx 1000
```

## Usage as a library

```
[dependencies]
fbas-reward-distributor = { version = "0.1", default-features = true }
```

See the [fbas-graph-generator](https://gitlab.informatik.hu-berlin.de/ti/theses/student-content/ndolo-charmaine-ma/fbas-graph-generator) for some examples.

## Performance and approximation measurements

1. Build with

```
cargo build --release --features "measurements"
```
2. then run performance measurements

```
target/release/performance_tests -m $MAX_TOP_TIER --no-quorum-intersection -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS -u $FBAS_TYPE
```
3. and/or approximation measurements

```
target/release/approximation_tests -m $MAX_TOP_TIER --no-quorum-intersection -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS -u $FBAS_TYPE
```
