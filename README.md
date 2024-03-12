# CoreScriptions Indexer

[![gitbook.io](https://img.shields.io/badge/docs-insdexer-pink?logo=GitBook)](https://insdexer.gitbook.io/insdexer)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](#license)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.75%2B-green)](#rust-version-requirements)
[![CI](https://github.com/insdexer/insdexer/actions/workflows/ci.yml/badge.svg)](https://github.com/insdexer/insdexer/actions/workflows/ci.yml)


## What is CoreScritions?
CoreScritions taking inspiration from Bitcoin's BRC-20 standard, CORE-20 inscriptions are unique CoreDao addresses that contain images or metadata stored directly on the blockchain. This token standard streamlines token creation and functionality within the network. CORE-20 enhances interoperability, simplifying token development and fostering a robust DeFi ecosystem.

## How do CoreScritions work?
As one of the first CORE-20 tokens that spearheaded this groundbreaking standard, COCO does everything a CORE-20 token can. This standard opens doors to diverse tokenized assets and use cases.
One innovative aspect of these tokens is their mutability. Contrary to the immutable nature of typical NFTs, CORE-20 tokens can be altered, allowing for dynamic changes like native trait swapping or implementing day/night cycles in gaming NFTs. This flexibility opens up new avenues for use cases in the digital art and gaming sectors. Another important aspect is their ranking system, which is based on the order of their creation, similar to Bitcoin Ordinals. This method creates a sense of urgency or FOMO (Fear of Missing Out), encouraging users to engage in the inscription process early. To inscribe an CORE-20 token, a user must possess update authority, which is usually bestowed upon the NFT's creator. This requirement ensures that control over the inscription process remains with the project owners, preventing unauthorized alterations to the inscribed image. However, the introduction of CORE-20 tokens is not without its challenges. Storing large files directly on the CORE blockchain can lead to issues like increased bandwidth requirements for validators and higher costs for infrastructure providers. Critics argue that given CORE's robust native capabilities for handling complex NFT transactions and storage, the addition of CORE-20 tokens might be unnecessary. They point out that, unlike Bitcoin, which needs BRC-20s due to its limited scripting capabilities, CORE can efficiently manage NFTs without requiring an additional layer like CORE-20. Despite these challenges, the innovation and possibilities brought forth by CORE-20 inscriptions are shaping a new future for digital art on CORE, offering a vision of secure, permanent, and fully integrated digital artworks within the blockchain network.

## What is Insdexer
CoreScriptions and CORE-20 tokens are powered by [Insdexer](https://insdexer.io), a collaborative effort of independent contributors and teams within the CORE ecosystem. They believe that the dynamic rendering and on-chain media support of CORE-20 tokens offer a more enriched and interactive digital art experience compared to existing solutions. Their commitment extends to maintaining an open-source framework and a policy of zero mint tax, making the platform more accessible for NFT creation and trade.

<!--ts-->

* [System Requirements](#system-requirements)
* [Usage](#usage)
  * [Getting Started](#getting-started)
  * [Configuration](#configuration)
  * [Logging](#logging)
  * [RESTful-API](#restful-api)

* [Key features](#key-features)
  * [Compatible Inscribe](#compatible-inscribe)
  * [Efficient Storage](#efficient-storage)
  * [Multi-Thread Sync](#multi-thread-sync)
  * [JSON-RPC daemon](#json-rpc-daemon)

* [Documentation](#documentation)
* [FAQ](#faq)
* [Getting in touch](#getting-in-touch)
  * [Reporting security issues/concerns](#reporting-security-issues/concerns)
  * [Team](#team)

<!--te-->

**Disclaimer**: this software is currently a tech preview. We will do our best to keep it stable and make no breaking changes but we don't guarantee anything. Things can and will break.

## System Requirements

* CPU: 4-core (or 2-core hyperthreaded).

* RAM: >=16GB, 64-bit architecture.

* Storage: > 500Gb.
    SSD or NVMe. Do not recommend HDD.
    Bear in mind that SSD performance deteriorates when close to capacity.

## Usage

### Getting Started

Example: for CoreScriptions on CoreDao network

```sh
./insdexer --web3-provider=https://rpc.coredao.org --start-block=9931503 --start-block-mint=9982899 --start-block-nft=9931503
```

### Configuration

```sh
./insdexer -h
Usage: insdexer [OPTIONS] --web3-provider <WEB3_PROVIDER> --start-block <START_BLOCK> --start-block-mint <START_BLOCK_MINT>

Options:
      --tick-max-len <TICK_MAX_LEN>
          The maximum length of tick [env: TICK_MAX_LEN=] [default: 32]
      --worker-count <WORKER_COUNT>
          The number of workers for sync blocks data [env: WORKER_COUNT=1] [default: 1]
      --confirm-block <CONFIRM_BLOCK>
          The number of confirm block, when inscribe a new block data [env: CONFIRM_BLOCK=1] [default: 1]
      --chain-id <CHAIN_ID>
          The chain id of the network [env: CHAIN_ID=1] [default: 1]
      --web3-provider <WEB3_PROVIDER>
          The web3 provider url [env: WEB3_PROVIDER=https://rpc.sepolia.org]
      --start-block <START_BLOCK>
          The start block number for sync and inscribe [env: START_BLOCK=5000000]
      --start-block-mint <START_BLOCK_MINT>
          The start block number for sync and token mint [env: START_BLOCK_MINT=5000000]
      --reindex
          Reindex the block data [env: REINDEX=]
      --worker-buffer-length <WORKER_BUFFER_LENGTH>
          The length of worker sync buffer [env: WORKER_BUFFER_LENGTH=100] [default: 64]
      --db-path <DB_PATH>
          The path of database [env: DB_PATH=./data] [default: ./data]
      --token-protocol <TOKEN_PROTOCOL>
          The token protocol [env: TOKEN_PROTOCOL=erc-20] [default: erc-20]
      --http-bind <HTTP_BIND>
          The rpc http bind address [env: HTTP_BIND=0.0.0.0] [default: 127.0.0.1]
      --http-port <HTTP_PORT>
          The rpc http port [env: HTTP_PORT=8888] [default: 8711]
      --api-only
          Run in api only mode [env: API_ONLY=]
      --open-files-limit <OPEN_FILES_LIMIT>
          The open files limit [env: OPEN_FILES_LIMIT=1024000] [default: 10240]
      --checkpoint-span <CHECKPOINT_SPAN>
          The checkpoint span [default: 10]
      --checkpoint-len <CHECKPOINT_LEN>
          The checkpoint length [default: 20]
      --finalized-block <FINALIZED_BLOCK>
          Finalized block [env: FINALIZED_BLOCK=] [default: 50]
      --checkpoint-path <CHECKPOINT_PATH>
          Checkpoint base path [env: CHECKPOINT_PATH=] [default: ./.checkpoint]
      --market-address-list <MARKET_ADDRESS_LIST>
          The market address list [default: ]
  -h, --help
          Print help
  -V, --version
          Print version
```

### Logging

log4rs.yaml

```yaml
refresh_rate: 30 seconds
appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d} {l} {m}{n}"
  file:
    kind: file
    path: "logs/insdexer.log"
    encoder:
      pattern: "{d} {l} {m}{n}"
root:
  level: info
  appenders:
    - stdout
    - file
```

## Key features

### Compatible Inscribe

For developers, just set token-protocol and modify marketplace module, a new inscriptions indexer born.

### Efficient Storage

Flat KV storage. Insdexer use RocksDB (a key-value database) and storage in a simple way.

Transaction Inscribe. Insdexer utilize transactions to batch process block data, aiming for high performance and consistency.

It has a Log-Structured-Merge-Database (LSM) design with flexible tradeoffs between Write-Amplification-Factor (WAF), Read-Amplification-Factor (RAF) and Space-Amplification-Factor (SAF). It has multi-threaded compactions, making it especially suitable for storing multiple terabytes of data in a single database.

### Multi-Thread Sync

Insdexer separate block syncing and inscribe. It use multi-threading for block data synchronization.
For first syncing, you can adjust the number of threads and working buffer according to machine configuration and bandwidth. After reaching the latest block, adjustments to settings can be made.

### JSON-RPC daemon

## Documentation

[https://insdexer.gitbook.io](https://insdexer.gitbook.io)

## FAQ

## Getting in touch

### Reporting security issues/concerns

### Team
