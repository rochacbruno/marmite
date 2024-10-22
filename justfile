check:
  cargo fmt -- --check
  cargo clippy -- -W clippy::pedantic

fmt:
  cargo fmt

fix:
  cargo clippy --fix -- -W clippy::pedantic
