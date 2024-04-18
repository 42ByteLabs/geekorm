#!/bin/bash
# This script bumps the version of the crate

LOCATION=Cargo.toml

# get the current version
current_version=$(grep -oP '^version = "(.*)"$' $LOCATION | cut -d '"' -f 2)
echo "📦 Current version :: $current_version"

IFS=. read -r major minor patch <<< "$current_version"

new_version="$major.$minor.$((patch + 1))"
echo "🚀 New version     :: $new_version (bump patch)"

# replace the version
sed -i "s/^version = \".*\"$/version = \"$new_version\"/" $LOCATION

if [ -z "$CI" ]; then
    echo "⚡ Updated '$LOCATION' with new version '$new_version'"
else
    echo "version=$new_version" >> $GITHUB_OUTPUT
fi
