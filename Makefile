
include build/common.mk

test:
	cargo test --all
	echo debug test: OK
	cargo test --all --release
	echo release test: OK
	cargo run -p tests

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
      -A clippy::module-name-repetitions \
      -A clippy::return-self-not-must-use \
      -A clippy::needless_pass_by_value \
      -A clippy::explicit_deref_methods \
      \
      -D warnings

