#!/bin/bash
# This script checks if the crate.io version is outdated (older than 2 weeks)

set -e

current_date=$(date +%s)
echo "⏲️  Current date   :: $(date -d @${current_date})"

crates_remote=$(curl -s https://crates.io/api/v1/crates/geekorm/versions | jq -r '.versions[0].created_at')
crates_date=$(date -d "${crates_remote}" +%s)
echo "🦀 Crates.io date :: $(date -d @${crates_date})"

diff=$((current_date - crates_date))

# show in minutes and seconds
echo "🔍 Difference     :: ${diff} (seconds)"
echo ""

# check if the difference is greater than 2 weeks
if [ $diff -gt 1209600 ]; then
    echo "🚀 The crate is outdated"
    if [ -z "$CI" ]; then
        exit 1
    else
        echo "changes=true" >> $GITHUB_STATE
    fi
else
    echo "👍 The crate is up to date"
    if [ -z "$CI" ]; then
        exit 0
    else
        echo "changes=false" >> $GITHUB_STATE
    fi
fi
