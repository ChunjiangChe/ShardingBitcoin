#!/bin/bash
cd ../..
./target/debug/bitcoin --p2p 10.0.0.2:0000 --api 10.0.0.2:0001 --shardId 0 --nodeId 0 --experNumber 4 --shardNum 3 --shardSize 3
