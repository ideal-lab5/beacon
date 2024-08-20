# Beacon

A simple centralized beacon relayer. It listens to randomness output from the Ideal Network and encodes pulses within a Substrate pallet (pallet-randomness-beacon);

## Run

### Docker

To run the beacon from docker

``` shell
docker pull ideallabs/relayer
docker run --network host ideallabs/relayer [node_websocket_URI]
```

For example, to connect to a locally running node whose RPC is exposed on 9944:

``` shell
docker run --network host ideallabs/relayer ws://127.0.0.1:9944
```

### Build from Sources

To build thje

1. (optional) install subxt
```
cargo install subxt-cli
```

2. (Optional) Generate metadata
The following is required only if you need to generate metadata (e.g. the runtime has changed)
0a. Run a local Ideal node
0b. Once it is running and producing ETF blocks, generate metadata by running `./generate_metadata.sh` from the root directory.

3. Run:

``` shell
cargo run [node_websocket_uri]
```

This will start the beacon relayer. If it successfully connects to the network, it will listen for justifications, interpolate them, and send the result as a signature to the beacon pallet.

## Testing

TODO