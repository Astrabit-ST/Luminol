#!/bin/sh
set -e

# Wait until Trunk errors out or builds successfully, then restore the old Cargo config
sleep 1
while [ -d $TRUNK_STAGING_DIR ] && [ ! -f $TRUNK_STAGING_DIR/luminol.js ] && pgrep -x 'trunk' > /dev/null; do
	sleep 1
done
mv .cargo/config.toml.bak .cargo/config.toml
