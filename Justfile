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

repl vault="jj_vault":
	cargo run -- repl --vault {{vault}}

embed-index vault="jj_vault":
	cargo run -- index --vault {{vault}}
