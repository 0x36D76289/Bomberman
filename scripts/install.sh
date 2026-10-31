#!/bin/bash
#
set -e
cd "$(dirname "$0")/.."

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$HOME/.local/bin/bomberman"
    rm -f "$HOME/.local/share/applications/bomberman.desktop"
    rm -f "$HOME/.local/share/icons/hicolor/64x64/apps/bomberman.png"
    exit 0
fi

install -Dm755 Bomberman "$HOME/.local/bin/bomberman"
install -Dm644 assets/textures/icon.png "$HOME/.local/share/icons/hicolor/64x64/apps/bomberman.png"

printf '[Desktop Entry]\nType=Application\nName=Bomberman\nExec=%s\nIcon=bomberman\nCategories=Game;\n' "$HOME/.local/bin/bomberman" > "$HOME/.local/share/applications/bomberman.desktop"

@echo "gros gros meow"
