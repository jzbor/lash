# Installation

## Building from Source
You need two things to build the project from source: [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and a C compiler/linker like [gcc](https://gcc.gnu.org/install/) or [clang](https://clang.llvm.org/get_started.html).
```
# Clone the source repository
git clone https://github.com/jzbor/lash.git
# Build release build with cargo
cargo build --release
# You can now find the binary in ./target/release
ls ./target/release
```
