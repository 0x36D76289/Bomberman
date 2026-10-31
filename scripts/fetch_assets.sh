#!/bin/bash

set -e

cd "$(dirname "$0")/.."
curl -L https://github.com/0x36D76289/Bomberman/releases/download/assets/assets.zip -o assets.zip

unzip -o assets.zip

@echo "mini meow"
