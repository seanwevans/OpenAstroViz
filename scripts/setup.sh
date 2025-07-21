#!/usr/bin/env bash
# OpenAstroViz setup script
# Installs Rust toolchain, CUDA toolkit, Node.js, and sets up pre-commit hooks

set -euo pipefail

# Functions
info() { echo -e "\033[1;34m[setup]\033[0m $1"; }

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
        info "Installing NVIDIA CUDA toolkit (apt)"
        sudo apt-get update
        sudo apt-get install -y nvidia-cuda-toolkit
    else
        info "CUDA toolkit already installed"
    fi
}

install_node() {
    if ! command -v node >/dev/null 2>&1; then
        info "Installing Node.js (apt)"
        sudo apt-get update
        sudo apt-get install -y nodejs npm
    else
        info "Node.js already installed"
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

install_rust
install_cuda
install_node
setup_precommit

info "Done. You may need to restart your shell for PATH changes to take effect."
