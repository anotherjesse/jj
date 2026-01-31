set dotenv-load := true
set shell := ["bash", "-cu"]

default: list

list:
	@just --list

verify:
	scripts/verify.sh

smoketest:
	scripts/smoke_openai.sh

vault-init path="jj_vault":
	cargo run -- vault init --path {{path}}

thread-create vault="jj_vault":
	cargo run -- thread create --vault {{vault}}

chat vault="jj_vault":
	cargo run -- chat --vault {{vault}}

embed-index vault="jj_vault":
	cargo run -- index --vault {{vault}}

gateway-start:
	cargo run -- gateway start

gateway-stop:
	cargo run -- gateway stop

gateway-status:
	cargo run -- gateway status

gateway-open:
	#!/usr/bin/env bash
	token=$(cat ~/.jj/gateway/token 2>/dev/null || echo "")
	port=${JJ_GATEWAY_PORT:-9123}
	open "http://127.0.0.1:${port}/?token=${token}"

install:
	cargo install --path . --root ~/.local
