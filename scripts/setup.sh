#!/usr/bin/env bash
# OpenAstroViz setup script
# Installs Rust toolchain, CUDA toolkit, Node.js, and sets up pre-commit hooks

set -euo pipefail

# Functions
info() { echo -e "\033[1;34m[setup]\033[0m $1"; }

# Detect operating system
OS_ID="unknown"
OS_LIKE=""
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS_ID="$ID"
    OS_LIKE="${ID_LIKE:-}"
else
    OS_ID="$(uname -s)"
fi

use_apt=false
if [[ "$OS_ID" == "debian" || "$OS_ID" == "ubuntu" || "$OS_LIKE" == *debian* || "$OS_LIKE" == *ubuntu* ]]; then
    use_apt=true
fi

install_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        info "Installing Rust via rustup"
        curl https://sh.rustup.rs -sSf | sh -s -- -y
        export PATH="$HOME/.cargo/bin:$PATH"
    else
        info "Rust already installed"
    fi
}

install_cuda() {
    if ! command -v nvcc >/dev/null 2>&1; then
        if $use_apt; then
            info "Installing NVIDIA CUDA toolkit (apt)"
            sudo apt-get update
            sudo apt-get install -y nvidia-cuda-toolkit
        else
            info "Please install the NVIDIA CUDA toolkit manually for your OS."
        fi
    else
        info "CUDA toolkit already installed"
    fi
}

install_node() {
    if ! command -v node >/dev/null 2>&1; then
        if $use_apt; then
            info "Installing Node.js (apt)"
            sudo apt-get update
            sudo apt-get install -y nodejs npm
        else
            info "Please install Node.js and npm manually for your OS."
        fi
    else
        info "Node.js already installed"
    fi
}

install_yarn() {
    if ! command -v yarn >/dev/null 2>&1; then
        info "Installing Yarn"
        npm install -g yarn
    else
        info "Yarn already installed"
    fi
}

setup_precommit() {
    if ! command -v pre-commit >/dev/null 2>&1; then
        info "Installing pre-commit"
        pip install --user pre-commit
    fi
    info "Installing git hooks"
    pre-commit install
}

info "Detected OS: $OS_ID"
install_rust
install_cuda
install_node
install_yarn
setup_precommit

info "Done. You may need to restart your shell for PATH changes to take effect."
