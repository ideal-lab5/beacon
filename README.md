# Beacon

> This is a WIP.

A simple centralized beacon contract and trusted relayer to interpolate signatures from the Ideal Network's justifications stream and make them available for on-chain protocols.

## Setup

1. install subxt
```
cargo install subxt-cli
```

2. (Optional) Generate metadata
The following is required only if you need to generate metadata (e.g. the runtime has changed)
0a. Run a local Ideal node
0b. Once it is running and producing ETF blocks, generate metadata by running `./generate_metadata.sh` from the root directory.

3. Run:

``` shell
cargo run
```

This will start the beacon. If it successfully connects to the network, it will listen for justification, interpolate them, and send the result as a signature to the beacon contract.

## Testing

TODO