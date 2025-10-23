#!/bin/bash
cd ../../../../../
sudo ./target/debug/powchain manifoldchain --p2p 127.0.0.1:6000 --api 127.0.0.1:7000 --shardId 0 --nodeId 0 --experNumber 0 --experIter 1 --shardNum 5 --shardSize 5 --blockSize 2048 --k 6 --domesticRatio 0.7 --eDiff 00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff --iDiff 000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffff