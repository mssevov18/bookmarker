#!/usr/bin/env sh

set -e

INSTALL_DIR="$HOME/.config/bookmarker"
BIN_SRC="target/release/bookmarker"
WRAPPERS_SRC="wrappers"

echo "Ensure install directory exists: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

echo "Copy binary to $INSTALL_DIR"
cp "$BIN_SRC" "$INSTALL_DIR/"

echo "Link binary in .local/bin"
mkdir -p "$HOME/.local/bin"
ln -sf "$INSTALL_DIR/bookmarker" "$HOME/.local/bin/bookmarker"

echo "Copy shell wrappers to $INSTALL_DIR"
cp -r "$WRAPPERS_SRC"/* "$INSTALL_DIR/"

echo "Install complete!"
echo "To use: source the appropriate shell wrapper, e.g.:"
echo "	source $INSTALL_DIR/bookmarker_wrapper.sh   # bash/zsh"
echo "	source $INSTALL_DIR/bookmarker_wrapper.fish # fish"
