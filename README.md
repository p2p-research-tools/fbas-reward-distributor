# FBAS Node Influence

Framework for the computation of a node's influence in a Federated Byzantine Agreement Systems (FBASs) like [Stellar](https://www.stellar.org/).

Build and run

```
cargo run --release -- test_data/mobilecoin_nodes_2021-10-22.json -d 10 -i noderank
```

The above will compute how 10 units should be distributed among the nodes in the `mobilecoin_nodes_2021-10-22.json` using the Noderank algorithm.
