ShardingBitcoin (aka `powchain`) is an experimental proof-of-work implementation that splits transaction processing across shards while using a global ordering chain to confirm shard outputs. The crate also keeps a plain, non-sharded bitcoin-style implementation for comparison.

## Top-Level Layout
- `src/main.rs` – CLI entry point (`cargo run sharding_bitcoin ...`) that wires configuration, networking, mining, and the HTTP API together.
- `src/types` – shared primitives: `H256` hashing helpers, Merkle tree, RocksDB-backed `Database` wrapper, key utilities, and a `Random` trait used by generators.
- `src/sharding_bitcoin` – the sharded protocol implementation (described below).
- `src/bitcoin` – legacy single-chain PoW implementation with matching modules (block, tx, miner, network, api) kept for reference but not invoked by default.
- `scripts/` – shell helpers to spin up local testnets; `scripts/sharding_bitcoin_test/*.sh` runs a 4-node, 2-shard demo with sensible defaults.

## ShardingBitcoin Module Map
- `configuration` – runtime knobs (difficulty targets for shard vs order blocks, block size, confirmation depth `k`, shard/node IDs, shard topology, experiment labels).
- `transaction` – UTXO-style transactions plus helpers to sign/verify (`ring::signature`), build initial balances, and consume UTXOs with flags describing cross-shard movement.
- `block` – data structures:
  - `BlockHeader` (parents for order/shard chains, Merkle root, timestamp, shard id),
  - `ShardBlock` (tx Merkle tree, PoW nonce), `OrderBlock` (confirmed shard block hashes), generic `Block` used as a pre-PoW bundle,
  - traits `Info`/`Content` for shared accessors.
  - `block/versa_block.rs` wraps either block type in `VersaBlock`/`VersaHash` so the rest of the code can treat them uniformly.
- `blockchain` – per-chain fork-aware storage. Uses `Database<VersaBlock>` (RocksDB) plus an in-memory tree of hashes to track heights, tips, confirmation depth, and forking rate metrics.
- `multichain` – orchestrator holding one ordering chain plus `shard_num` shard chains. Updates confirmed shard blocks (those `k` deep) and exposes accessors used by the miner, API, and network.
- `mempool` – RocksDB-backed transaction buffer with a queue for block assembly.
- `miner` – PoW engine. Builds a candidate hybrid block from current order/shard parents and locally available transactions, then mines:
  - if `pow_hash` meets `order_diff`, emit an `OrderBlock`;
  - else if it meets `block_diff`, emit a `ShardBlock`;
  mined blocks are sent over a channel to the miner worker.
  - `miner/worker` inserts self-mined blocks into `Multichain` and broadcasts them.
- `network` – lightweight P2P layer:
  - `message` defines wire enums (`NewBlockHash`, `GetBlocks`, `Blocks`, `Ping/Pong`).
  - `server` listens for peers, exchanges shard IDs on connect, and manages peer write queues.
  - `peer` holds per-connection write state.
  - `worker` decodes inbound frames, requests missing parents, validates/inserts blocks into `Multichain`, and rebroadcasts newly accepted hashes.
- `api` – HTTP server (tiny_http) exposing:
  - `/miner/start?lambda=<micros>` and `/miner/end` to control mining,
  - `/network/ping` to broadcast a ping,
  - `/blockchain/ordering-chain`, `/blockchain/shard-chain`, `/blockchain/shard-chain-with-shard?shard-id=X` to inspect longest-chain hashes and forking rates.

## How Components Fit Together
1) `cargo run sharding_bitcoin ...` calls `sharding_bitcoin::start`, parses CLI flags, builds a `Configuration`, and seeds genesis order/shard chains inside `Multichain` plus a RocksDB-backed `Mempool`.
2) A P2P `server` starts (listening on `--p2p`) and hands incoming frames to `network::worker` threads (`--p2p-workers`). Workers gossip block hashes, request missing parents, and insert verified blocks into `Multichain`.
3) `miner` threads watch chain tips via `Multichain`, assemble transactions from `Mempool`, mine shard/order blocks according to the two difficulty thresholds, and send results to the miner worker. The worker inserts the block locally and broadcasts it.
4) The `api` server (`--api`) provides knobs for starting/stopping mining and endpoints for chain inspection; it queries `Multichain` for chain state and uses the network handle for ping tests.
5) Persistence: `Database<T>` instances in blockchain/mempool store data under `./DB/node(shard-<id>,index-<id>)/...`, so each node keeps an isolated RocksDB namespace.

## Running a Local Demo
- Prereqs: Rust toolchain, no external services. Networking is local-only by default.
- Quick start: open multiple terminals and run the helper scripts in `scripts/sharding_bitcoin_test/` (e.g., `node1.sh` … `node4.sh`) to launch a 4-node, 2-shard network with preset ports and difficulties.
- Manual run example (shard 0, node 0):
  ```bash
  cargo run sharding_bitcoin \
    --p2p 127.0.0.1:6000 --api 127.0.0.1:7000 \
    --shardId 0 --nodeId 0 --shardNum 2 --shardSize 2 \
    --blockSize 2048 --k 7 \
    --bDiff 3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3 \
    --oDiff 10c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c3
  ```
  Add multiple `-c/--connect` peers to join nodes together.
- Query chains: hit `http://127.0.0.1:7000/blockchain/ordering-chain` (or `.../shard-chain`) after the node starts; start mining via `.../miner/start?lambda=0`.

## Notes
- The `src/bitcoin` modules mirror the sharded layout for a single-chain baseline; keep them in sync if you extend core primitives.
- Data and logs accumulate under `./DB` and `./log`; remove them between experiments if you want a fresh state.
