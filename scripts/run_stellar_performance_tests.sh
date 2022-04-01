#!/bin/sh
source $HOME/.bashrc

BIN_DIR=$HOME/thesis/fbas_reward_distributor

JOBS=8
OUTPUT_FILE=results/ranking_algorithms_performance_measurements_stellar_2022_03_29.csv
FBAS_TYPE=stellar
ITERATIONS=10
MAX_TOP_TIER=35

LOG_FILE=logs/stellar_performance_tests_2022_03_29.log

cd $BIN_DIR
mkdir logs
mkdir results

cargo build --release --bin performance_tests --features "measurements"
#target/release/performance_tests -m $MAX_TOP_TIER  -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS -u $FBAS_TYPE
target/release/performance_tests -m $MAX_TOP_TIER  -r $ITERATIONS -o $OUTPUT_FILE -j $JOBS -u $FBAS_TYPE > $LOG_FILE 2>&1
