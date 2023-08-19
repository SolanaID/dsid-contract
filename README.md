# DSID Smart Contract

## Smart Contract functions

- ### [Minting Tokens: Adding a new Reputation Category](.//src/contract/add.rs)

    *Only the owner of the Contract (Backend) will be able to perform this operation*

- ### [Minting Token: with following fields for each token](.//src/contract/mint.rs)

    *Only the owner of the Contract (Backend) will be able to perform this operation*
  - Token Id (Reputation Category).
  - Token Quantity (Reputation Score).
  - Expiration Time
  - Account Address

- ### [Updating Token Metadata](.//src/contract/token_metadata.rs)

    Metadata details to be finalized later but it should contain information to calculate grade based on the function of Reputation Score which frontend apps can use to display a grade instead of Reputation score. Only the owner of the Contract (Backend) will be able to perform this operation

- ### [Check Token Balance](.//src/contract/balance_of.rs)

    (Checking a specified reputation score for a specified account address).*Anyone can read this information*

- ### [Retrieving Token Metadata URL](.//src/contract/token_metadata.rs) : Standard Implementation as per CIS2 standards

    *Anyone can read this information*.

- ### Contract will not implement the following CIS2 functions and will return a non supported error

  - [Transfer](.//src/contract/transfer.rs)
  - [Update Operator](.//src/contract/update_operator.rs)
  - [Operator Of](.//src/contract/operator_of.rs)

## Pre-requisites

- [Concordium Client](https://developer.concordium.software/en/mainnet/net/installation/downloads.html#concordium-client-client-version)
- [Cargo Concordium](https://developer.concordium.software/en/mainnet/net/installation/downloads.html#cargo-concordium-v2-8-0)
- [Concordium Node](https://developer.concordium.software/en/mainnet/net/nodes/node-requirements.html)

## Build Smart Contract

```bash
cargo concordium build --out ./module.wasm --schema-out ./schema.bin --schema-embed
```

## Test Smart Contract

```bash
cargo concordium test
```

## Deploy Smart Contract

- [Setup Concordium Client](https://github.com/ivanmolto/concordium-setup)
- Deploy Smart Contract (See [deploy.sh](./deploy.sh))

    ```bash
    concordium-client --grpc-ip $CONNCORDIUM_NODE_ENDPOINT module deploy ./module.wasm --sender $SENDER --no-confirm
    ```
