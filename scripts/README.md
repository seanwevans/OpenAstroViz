# Scripts module

The `setup.sh` script bootstraps the development environment. It installs the Rust compiler, CUDA toolkit, Node.js, Yarn and configures pre-commit hooks. The script first detects your operating system via `/etc/os-release` or `uname`.

On Debian or Ubuntu systems it will automatically install dependencies with `apt-get`. macOS users with Homebrew will have packages installed via `brew`. On systems without a supported package manager the script prints a message asking you to install the prerequisites manually.

Run it from the repository root:

```bash
./scripts/setup.sh
```

## Testing

The `run_tests.sh` script compiles and runs the C++ unit tests. Ensure the
Catch2 library is installed (for example via `apt-get install catch2`) and
execute:

```bash
./scripts/run_tests.sh
```

## Windows

`setup.sh` does not currently automate installation on Windows. Install the following tools manually:

1. [Rust toolchain](https://www.rust-lang.org/tools/install)
2. [Node.js](https://nodejs.org/)
3. [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) (optional for GPU support)

After installing, ensure `cargo`, `node` and optionally `nvcc` are in your `PATH`. Finally run `pre-commit install` to set up the git hooks.
