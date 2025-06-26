#!/bin/bash

# get-basic-image-tag.sh
# Get tag for basic image builds - either from TAG env var or from git tag

set -e

# Check if TAG is provided as environment variable
if [ -n "$TAG" ] && [ "$TAG" != "" ]; then
    echo "Using provided TAG: $TAG"
    export BASIC_IMAGE_TAG="$TAG"
else
    echo "No TAG provided, getting git tag..."

    # Source the existing get-git-tag script
    source "$(dirname "$0")/get-git-tag.sh"

    # Use the git tag
    export BASIC_IMAGE_TAG="$GIT_TAG"
    echo "Using git tag: $BASIC_IMAGE_TAG"
fi

# Make the tag available for other scripts
echo "Basic image tag set to: $BASIC_IMAGE_TAG"
