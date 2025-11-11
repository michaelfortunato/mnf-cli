
build:
  cargo build --release

# attr -l ~/bin/rune and clear
#          com.apple.quarantin
install: build
  rm ~/bin/mnf
  cp target/release/mnf-cli ~/bin/mnf
