#!/bin/bash
start_server_pid=$(pgrep -f "generate_node_start.sh")
start_miner_generator_pid=$(pgrep -f "start_all_miner_generator.sh")
kill $start_miner_generator_pid
sleep 1
kill $start_server_pid
sleep 1
pkill -f "target/debug/bitcoin"
