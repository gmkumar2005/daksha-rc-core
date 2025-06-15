#!/bin/bash
# Get the latest Git tag, fallback to "latest" if no tags exist
GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "latest")
echo "Latest Git tag: $GIT_TAG"
export GIT_TAG