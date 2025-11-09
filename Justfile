
build:
  cargo build --release

install: build
  cp target/release/mnf-cli ~/bin/mnf
