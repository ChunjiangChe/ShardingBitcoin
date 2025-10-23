#!/bin/bash

shard_num=5
shard_size=5
exper_number=0
iter=1

cd ../../../
./end_node.sh $shard_num $shard_size
sleep 10