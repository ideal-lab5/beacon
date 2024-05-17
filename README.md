# Beacon

> This is a WIP.

A simple centralized beacon contract and trusted relayer to interpolate signatures from the Ideal Network's justifications stream and make them available for on-chain protocols.

## Setup

(Optional)
The following is required only if you need to generate metadata (e.g. the runtime has changed)
0a. Run a local Ideal node
0b. Once it is running and producing ETF blocks, generate metadata by running `./generate_metadata.sh` from the root directory.

Either connecting to a local node or hosted one:
1. Deploy the beacon contract and use the resulting address to get a public key (either with subkey or ss58.org)
2. Insert the public key into the main.rs file (near the top), then execute:

``` shell
cargo run
```

This will start the beacon. If it successfully connects to the network, it will listen for justification, interpolate them, and send the result as a signature to the beacon contract.

## Testing

TODO