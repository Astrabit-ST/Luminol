#!/bin/sh
set -e

git_version=$(git describe --always --dirty=-modified)

# Print build information to buildinfo.json in the root directory of the output folder
# You can change the "epoch" if you need to make backwards-incompatible changes to the build info
echo "{\"epoch\":0,\"rev\":\"$git_version\",\"profile\":\"$TRUNK_PROFILE\"}" > $TRUNK_STAGING_DIR/buildinfo.json

# Enable std support for multithreading and set the LUMINOL_VERSION environment variable
[ ! -f $TRUNK_SOURCE_DIR/.cargo/config.toml.bak ] || mv $TRUNK_SOURCE_DIR/.cargo/config.toml.bak $TRUNK_SOURCE_DIR/.cargo/config.toml
cp $TRUNK_SOURCE_DIR/.cargo/config.toml $TRUNK_SOURCE_DIR/.cargo/config.toml.bak

echo "LUMINOL_VERSION = { value = \"$git_version\", force = true }" >> $TRUNK_SOURCE_DIR/.cargo/config.toml

echo '[unstable]' >> $TRUNK_SOURCE_DIR/.cargo/config.toml
echo 'build-std = ["std", "panic_abort"]' >> $TRUNK_SOURCE_DIR/.cargo/config.toml
