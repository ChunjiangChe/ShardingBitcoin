#!/bin/bash

# create the basic command
CMD="./powchain $PROTOCOL --p2p $P2P --api $API"

# if there are any peers, add them to the command
if [ ! -z "$PEERS" ]; then
    IFS=',' read -ra PEERS_ARRAY <<< $PEERS
    for PEER in ${PEERS_ARRAY[@]}; do
      CMD="$CMD -c $PEER"
    done
fi

# add other parameters
if [ "$PROTOCOL" == "optchain" ]; then
    CMD="$CMD --shardId $SHARD_ID --nodeId $NODE_ID --experNumber $EXPER_NUMBER --experIter $EXPER_ITER --shardNum $SHARD_NUM --shardSize $SHARD_SIZE --blockSize $BLOCK_SIZE --symbolSize $SYMBOL_SIZE --propSize $PROP_SIZE --avaiSize $AVAI_SIZE --eReq $EX_REQ_NUM --iReq $IN_REQ_NUM --k $K --tDiff $TX_DIFF --pDiff $PROP_DIFF --aDiff $AVAI_DIFF --iDiff $IN_AVAI_DIFF"
elif [ "$PROTOCOL" == "manifoldchain" ]; then
    CMD="$CMD --shardId $SHARD_ID --nodeId $NODE_ID --experNumber $EXPER_NUMBER --experIter $EXPER_ITER --shardNum $SHARD_NUM --shardSize $SHARD_SIZE --blockSize $BLOCK_SIZE --k $K --domesticRatio $DOMESTIC_RATE --eDiff $EX_DIFF --iDiff $IN_DIFF"
fi


# run the command
echo "Running command: $CMD"
exec $CMD
