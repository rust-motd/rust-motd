#!/bin/sh

# Generate example output image shown in readme
# Write output of rust-motd to file,
# then use termtosvg to convert this into an svg

# Temporary file for holding motd text (with escape sequences)
TXT_FILE="/tmp/rust-motd.txt"

SVG_FILE="/tmp/rust-motd-termtosvg/termtosvg_00000.svg"

cd $(dirname $0)

pwd

cargo build

sudo ../target/debug/rust-motd ./example_config.toml > $TXT_FILE

echo "" >> $TXT_FILE
(tput setaf 4; echo "~/code/rust_motd") >> $TXT_FILE
(tput setaf 2; echo -n "❯ "; tput setaf 7) >> $TXT_FILE

termtosvg \
	--still-frames \
	--template window_frame \
	--command "cat $TXT_FILE" \
	--screen-geometry 80x41 $(dirname $SVG_FILE)

cp $SVG_FILE ./example_output.svg
