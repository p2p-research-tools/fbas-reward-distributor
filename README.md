# FBAS Reward Distributor

![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
[![CI](https://github.com/cndolo/fbas-reward-distributor/actions/workflows/test.yml/badge.svg)](https://github.com/cndolo/fbas-reward-distributor/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/cndolo/fbas-reward-distributor/branch/main/graph/badge.svg?token=QZH345MHCJ)](https://codecov.io/gh/cndolo/fbas-reward-distributor)
[![dependency status](https://deps.rs/repo/github/cndolo/fbas-reward-distributor/status.svg)](https://deps.rs/repo/github/cndolo/fbas-reward-distributor)

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

- A minimal gcc and g++ toolchain is required by some of the dependencies. Should be covered by the
  `build-essential, m4` packages.

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

1. Running with cargo

```
cargo run --release
reward_distributor 0.1.0
Charmaine Ndolo
Rank nodes of an FBAS and allocate rewards to them accordingly

USAGE:
    reward_distributor <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    distribute    Compute a distribution based on ranking according to selected algorithm
    help          Prints this message or the help of the given subcommand(s)
    rank          Rank only, do not compute a distribution
```

The rank subcommand is similar to distribute with the exception that it only
calculates the nodes' weights without allocating rewards.

```
reward_distributor-rank 0.1.0
Charmaine Ndolo
Rank only, do not compute a distribution

USAGE:
    reward_distributor rank [FLAGS] [nodes-path] <SUBCOMMAND>

FLAGS:
    -n, --no-quorum-intersection    Do not assert that the FBAS has quorum intersection before proceeding with further
                                    computations. Default behaviour is to always check for QI
    -h, --help                      Prints help information
    -i, --ignore-inactive-nodes     Prior to any analysis, filter out all nodes marked as `"active" == false` in the
                                    input nodes JSON (the one at `nodes_path`)
    -p, --pretty                    Identify nodes by their public key. Default is to use node IDs corresponding to
                                    indices in the input file
    -V, --version                   Prints version information

ARGS:
    <nodes-path>    Path to JSON file describing the FBAS in stellarbeat.org "nodes" format. Will use STDIN if
                    omitted

SUBCOMMANDS:
    help                  Prints this message or the help of the given subcommand(s)
    node-rank             Use NodeRank, an extension of PageRank, to measure nodes' weight in the FBAS
    power-index-approx    Approximate Shapley values as a measure of nodes' importance in the FBAS. The number of
                          samples to use must be passed if selected. Optionally pass a seed that will be used by the
                          RNG
    power-index-enum      Use Shapley-Shubik power indices to calculate nodes' importance in the FBAS. Not
                          recommended for FBAS with many players because of time complexity
```

The rank subcommand is similar to distribute with the exception that it only calculates the nodes' weights without allocating rewards.
The output is always a sorted list of tuples: (NodeID, Public Key (where available), Ranking, [Reward]).

2. Compute a reward distribution for the nodes in the `mobilecoin_nodes_2021-10-22.json` FBAS using

    1. the Shapley-Shubik power index

        ```
            cargo run --release -- rank test_data/mobilecoin_nodes_2021-10-22.json power-index-enum
        ```

        This algorithm computes the players' Shapley-Shubik indices via enumeration in `O(2^n)` time, and is therefore not recommended for larger FBASs.

    2. As an alternative, we provide a polynomial time approximation algorithm using [Castro et al.'s algorithm](https://www.sciencedirect.com/science/article/abs/pii/S0305054808000804) based on sampling.

        ```
        cargo run --release -- rank test_data/mobilecoin_nodes_2021-10-22.json power-index-approx 1000
        ```

    3. Distributions can also be computed based on a graph-theoretic (NodeRank) metric:

        ```
        cargo run --release -- rank test_data/mobilecoin_nodes_2021-10-22.json node-rank

        ```

## Usage as a library

```
[dependencies]
fbas-reward-distributor = { git = "https://github.com/cndolo/fbas-reward-distributor", default-features = true}
```

See the [fbas-graph-generator](https://github.com/cndolo/fbas-graph-generator) for some examples.

## Performance and approximation measurements

1. Build with

```
cargo build --release --features "measurements"
```
2. then run performance measurements using the selected ranking algorithm

```
target/release/performance_tests -m $MAX_TOP_TIER --no-quorum-intersection -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS -u $FBAS_TYPE $RANKING_ALGO
```
3. and/or approximation measurements

```
target/release/approximation_tests -m $MAX_TOP_TIER --no-quorum-intersection -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS -u $FBAS_TYPE
```
