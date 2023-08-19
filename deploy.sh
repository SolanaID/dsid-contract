export CONNCORDIUM_NODE_ENDPOINT="127.0.0.1"
export SENDER="new"

cargo concordium build --out ./module.wasm --schema-out ./schema.bin --schema-embed
xxd -c 10000 -p ./schema.bin > schema.hex
concordium-client --grpc-ip $CONNCORDIUM_NODE_ENDPOINT module deploy ./module.wasm --sender $SENDER --no-confirm
