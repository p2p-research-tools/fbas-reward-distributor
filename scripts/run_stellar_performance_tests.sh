#!/bin/sh

BIN_DIR=$HOME/thesis/fbas_reward_distributor
JOBS=8
OUTPUT_FILE=ranking_algorithms_performance_measurements_stellar_2022_03_26.csv
FBAS_TYPE=stellar
ITERATIONS=10
MAX_TOP_TIER=35

cd $BIN_DIR
mkdir logs
cargo build --release --bin performance_tests --features "measurements"
target/release/performance_tests -m $MAX_TOP_TIER  -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS $FBAS_TYPE >& logs/stellar_performance_tests_2022_03_26.log
