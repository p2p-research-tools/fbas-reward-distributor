# FBAS Reward Distributor

Framework for the computation of a node's influence and reward distribution in
a Federated Byzantine Agreement Systems (FBASs) like
[Stellar](https://www.stellar.org/).

## 1. Required tools

    - A working [Rust](https://www.rust-lang.org) environment
        - Install: https://www.rust-lang.org/tools/install

## 2. Build and run
Compilation and execution can be achieved in a single step as shown below

```
cargo run --release -- SUBCOMMAND where subcommand is either 'distribute' or 'rank'.

cargo run --release -- distribute -a alg -r reward -p -i fbas-path
    - -a alg: algorithm to use to determine node rankings, can be 'noderank', 'exact-powerindex' or 'approx-powerindex'. Must be passed.
    - -r reward: reward value that is to be distributed. Default = 1.
    - -p: Include the nodes' public keys in the output. Default = false.
    - fbas-path: Path to file describing the FBAS. If no path is passed, the program will attempt to read from the command line..
    - -i: Ignore inactive nodes in the FBAS. Default = false.

The `rank` subcommand works in the same way except that -r is omitted without a default value.
```

## Computing a distribution for an FBAS

For example:

```
cargo run --release -- distribute -a noderank -r 10 test_data/mobilecoin_nodes_2021-10-22.json
```

will compute how 10 units should be distributed among the nodes in the `mobilecoin_nodes_2021-10-22.json` using a graph-theoretic (Noderank) metric.

The same can be accomplished using a game-theoretic concept, i.e. the 'Shapley - Shubik Power Index'.

```
cargo run --release -- distribute -a exact-powerindex -r 10 test_data/mobilecoin_nodes_2021-10-22.json
```

The exact implementation computes the players' exact Shapley-Shubik indices via enumeration in `O(2^n)` time, and is therefore not recommended for larger FBASs.

As an alternative, we provide an polynomial time approximation implementation using [Castro et al.'s algorithm](https://www.sciencedirect.com/science/article/abs/pii/S0305054808000804) based on sampling. 

```
cargo run --release -- distribute -a approx-powerindex -r 10 test_data/mobilecoin_nodes_2021-10-22.json
```

The output is a sorted list of tuples: (NodeID, Public Key (where available), Ranking, Reward).

## Ranking the nodes

The tool also supports calculating rankings alone using the implemented metrics via the `rank` subcommand.

```
cargo run --release -- rank -a exact-powerindex -p test_data/mobilecoin_nodes_2021-10-22.json
```

The output is a sorted list of tuples: (NodeID, Public Key (where available), Ranking).
