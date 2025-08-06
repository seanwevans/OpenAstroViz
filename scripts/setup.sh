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
use_brew=false

if [[ "$OS_ID" == "debian" || "$OS_ID" == "ubuntu" || "$OS_LIKE" == *debian* || "$OS_LIKE" == *ubuntu* ]]; then
    if command -v apt-get >/dev/null 2>&1; then
        use_apt=true
    else
        info "Warning: apt-get not found. Packages must be installed manually."
    fi
elif [[ "$OS_ID" == "Darwin" || "$OS_ID" == "darwin" ]]; then
    if command -v brew >/dev/null 2>&1; then
        use_brew=true
    else
        info "Warning: Homebrew not found. Packages must be installed manually."
    fi
fi

install_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        if $use_apt; then
            info "Installing Rust via rustup"
            curl https://sh.rustup.rs -sSf | sh -s -- -y
        elif $use_brew; then
            info "Installing Rust via Homebrew"
            brew install rustup-init
            rustup-init -y
        else
            info "Please install Rust manually for your OS."
        fi
        # Source cargo environment so rustup and cargo are in PATH for subsequent commands
        [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
    else
        info "Rust already installed"
    fi
    info "Setting local Rust toolchain to stable"
    rustup override set stable
}

install_cuda() {
    if ! command -v nvcc >/dev/null 2>&1; then
        if $use_apt; then
            info "Installing NVIDIA CUDA toolkit (apt)"
            sudo apt-get update
            sudo apt-get install -y nvidia-cuda-toolkit
        elif $use_brew; then
            if brew info cuda >/dev/null 2>&1; then
                info "Installing NVIDIA CUDA toolkit (brew)"
                brew install --cask cuda || brew install cuda
            else
                info "CUDA formula not found; skipping CUDA installation"
            fi
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
        elif $use_brew; then
            info "Installing Node.js (brew)"
            brew install node
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
