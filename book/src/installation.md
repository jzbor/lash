# Installation

## Building from Source
You need two things to build the project from source: [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and a C compiler/linker like [gcc](https://gcc.gnu.org/install/) or [clang](https://clang.llvm.org/get_started.html).
```sh
# Clone the source repository
git clone https://github.com/jzbor/lash.git
# Build release build with cargo
cargo build --release
# You can now find the binary in ./target/release
ls ./target/release
```

## Running as Nix Flake
If you have [Nix installed](https://nixos.org/download/) and [Flakes enabled](https://nix-tutorial.gitlabpages.inria.fr/nix-tutorial/flakes.html), installing `lash` should be as easy as:
```sh
nix profile install github:jzbor/lash
```
If you are using NixOS or home-manager you probably know how to install it best yourself.

You can also run the program without permanently installing it like so:
```sh
nix profile install github:jzbor/lash
```
