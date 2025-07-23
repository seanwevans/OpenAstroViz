# Scripts module

The `setup.sh` script bootstraps the development environment. It installs the Rust compiler, CUDA toolkit, Node.js, Yarn and configures pre-commit hooks. The script first detects your operating system via `/etc/os-release` or `uname`.

On Debian or Ubuntu systems it will automatically install dependencies with `apt-get`. On any other operating system the script prints a message asking you to install these prerequisites manually.

Run it from the repository root:

```bash
./scripts/setup.sh
```
