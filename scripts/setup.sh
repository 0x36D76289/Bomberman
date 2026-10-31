#!/bin/bash

set -e

sudo apt update
sudo apt install -y build-essential curl unzip pkg-config cmake libasound2-dev libudev-dev libvulkan-dev

curl -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
make release

@echo "big meow"
