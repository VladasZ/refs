
include build/common.mk

test:
	cargo test --all --all-features
	echo stats tests: OK
	cargo test --all
	echo debug tests: OK
	cargo test --all --all-features --release
	echo release tests: OK
	cargo run -p tests
	make test-wasm
	echo wasm tests: OK

test-wasm:
	cargo install wasm-pack
	cd refs && wasm-pack test --firefox --headless

lint:
	cargo clippy \
      -- \
      \
      -W clippy::all \
      -W clippy::pedantic \
      \
      -A clippy::module_inception \
      -A clippy::must-use-candidate \
      -A clippy::missing-errors-doc \
      -A clippy::missing-panics-doc \
      -A clippy::missing-safety-doc \
      -A clippy::module-name-repetitions \
      -A clippy::return-self-not-must-use \
      -A clippy::needless_pass_by_value \
      -A clippy::explicit_deref_methods \
      \
      -D warnings

