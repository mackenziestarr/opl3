# opl3
messing around with programing the Yamaha OPL3 with rust

# building

```console
rustup install nightly
rustup component add --target thumbv7em-none-eabi rust-std --toolchain=nightly
brew install gcc-arm-none-eabi # OSX
rustup override set nightly
rustup component add rust-src
rustup target add thumbv7m-none-eabi
make
make flash
```

# links
- https://branan.github.io/teensy/
- https://rust-embedded.github.io/book/
- https://github.com/rust-embedded/awesome-embedded-rust
- https://www.fit.vutbr.cz/~arnost/opl/opl3.html
- http://map.grauw.nl/resources/sound/yamaha_ymf262.pdf
