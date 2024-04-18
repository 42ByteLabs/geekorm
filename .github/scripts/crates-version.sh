#!/bin/bash
# This script checks if the crate.io version is outdated (older than 2 weeks)

set -e

LOCATION=Cargo.toml

current_version=$(grep -oP '^version = "(.*)"$' $LOCATION | cut -d '"' -f 2)
echo "💻 Current version   :: $current_version"

crates_remote=$(curl -s https://crates.io/api/v1/crates/geekorm/versions | jq -r '.versions[0].num')
echo "🦀 Crates.io version :: $crates_remote"

echo ""

if [ "$current_version" != "$crates_remote" ]; then
    echo "🚀 The crate is outdated... Let's update it!"
    echo "outdated=true" >> $GITHUB_STATE
else
    echo "🍹 Crate is up to date. Lets sit back and relax..."
    echo "outdated=false" >> $GITHUB_STATE
fi
