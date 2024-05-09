# Gateway

The gateway contract is a type of light client that lives on the target chain. The relayer provides signatures and proofs to the light client.

## Build

`cargo contract build`

## Test

`cargo contract test`

## Deploy

``` shell
cargo contract instantiate ./target/ink/gateway.contract --constructor new --args 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --suri //Alice --url ws://127.0.0.1:9944 -x
```