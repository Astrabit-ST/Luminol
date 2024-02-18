#!/bin/sh
set -e

git_version=$(git describe --always --dirty=-modified)
echo $git_version > $TRUNK_STAGING_DIR/git-rev.txt

# Enable std support for multithreading and set the LUMINOL_VERSION environment variable
[ ! -f $TRUNK_SOURCE_DIR/.cargo/config.toml.bak ] || mv $TRUNK_SOURCE_DIR/.cargo/config.toml.bak $TRUNK_SOURCE_DIR/.cargo/config.toml
cp $TRUNK_SOURCE_DIR/.cargo/config.toml $TRUNK_SOURCE_DIR/.cargo/config.toml.bak

echo '[env]' >> $TRUNK_SOURCE_DIR/.cargo/config.toml
echo "LUMINOL_VERSION = { value = \"$git_version\", force = true }" >> $TRUNK_SOURCE_DIR/.cargo/config.toml

echo '[unstable]' >> $TRUNK_SOURCE_DIR/.cargo/config.toml
echo 'build-std = ["std", "panic_abort"]' >> $TRUNK_SOURCE_DIR/.cargo/config.toml
