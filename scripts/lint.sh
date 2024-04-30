#!/bin/bash
set -eox pipefail

rustup component add clippy

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
