#!/bin/sh

#BIN_DIR=$HOME/thesis/fbas_reward_distributor
BIN_DIR=fbas_reward_distributor
JOBS=4
OUTPUT_FILE=results/ranking_algorithms_performance_measurements_stellar_2022_03_29.csv
FBAS_TYPE=stellar
ITERATIONS=1
MAX_TOP_TIER=35

cd $BIN_DIR
mkdir logs
cargo build --release --bin performance_tests --features "measurements"
target/release/performance_tests -m $MAX_TOP_TIER  -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS -u $FBAS_TYPE >& logs/stellar_performance_tests_2022_03_29.log
#target/release/performance_tests -m $MAX_TOP_TIER  -r $ITERATIONS -o $OUTPUT_FILE -u -j $JOBS $FBAS_TYPE
