build:
	cargo wasm

optimize:
	docker run --rm -v "$$(pwd)":/code \
		--mount type=volume,source="$$(basename "$$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.14.0
test:
	cargo unit-test

upload-testnet:
	seid tx wasm store ./artifacts/pierprotocol_sei.wasm -y --from=hk --chain-id=atlantic-2 --node https://rpc.atlantic-2.seinetwork.io --gas=10000000 --fees=1000000usei --broadcast-mode=block

instantiate-testnet:
	seid tx wasm instantiate ${id} '{"count": "${count}"}' --chain-id atlantic-2 --from hk --gas=4000000 --fees=1000000usei --broadcast-mode=block --label pierprotocol --no-admin --node https://rpc.atlantic-2.seinetwork.io

balance-hk-testnet:
	seid q bank balances sei1cz56s8l9yz92jgstv9y4pyxj8vkdnw7acug8n7 --node https://rpc.atlantic-2.seinetwork.io --chain-id atlantic-2
