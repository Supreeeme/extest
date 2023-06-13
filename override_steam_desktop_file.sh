#!/bin/bash

DIR="$(dirname $(readlink -f "$0"))"
EXTEST=$DIR/target/i686-unknown-linux-gnu/release/libextest.so

if ! [ -f $EXTEST ]; then
	echo "Extest not built, building..."
	cd $DIR
	cargo build --release
	if ! [ $? -eq 0 ]; then
		echo "Failed to build, aborting"
		exit 1
	fi
fi

STEAM_DESKTOP_FILE=""
IFS=: read -a search_paths < <(echo "$XDG_DATA_DIRS")
for path in "${search_paths[@]}"; do
	if [ -f $path/applications/steam.desktop ]; then
		STEAM_DESKTOP_FILE=$path/applications/steam.desktop
		break
	fi
done

if ! [ -f $STEAM_DESKTOP_FILE ]; then
	echo "Could not find Steam's desktop file, is XDG_DATA_DIRS	set properly?"
	exit 1
fi

DATA_PATH="${XDG_DATA_HOME:-$HOME/.local/share}"
NEW_DESKTOP_FILE="$DATA_PATH"/applications/steam.desktop
echo "Found Steam's desktop file at $STEAM_DESKTOP_FILE, copying to $NEW_DESKTOP_FILE"

set -e
mkdir -p "$(dirname $NEW_DESKTOP_FILE)"
cp $STEAM_DESKTOP_FILE $NEW_DESKTOP_FILE
sed -i "s,Exec=/usr/bin/steam,Exec=env LD_PRELOAD=$EXTEST /usr/bin/steam," $NEW_DESKTOP_FILE
echo "Extest has been set up, enjoy!"
