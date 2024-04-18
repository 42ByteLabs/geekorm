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
    echo "🚀 The crate is outdated"
    if [ -z "$CI" ]; then
        exit 1
    else
        echo "outdated=true" >> $GITHUB_STATE
    fi
else
    echo "🍹 Update to date. Lets sit back and relax..."
    if [ -z "$CI" ]; then
        exit 0
    else
        echo "outdated=false" >> $GITHUB_STATE
    fi
fi
