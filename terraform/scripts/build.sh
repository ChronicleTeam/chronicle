#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <git-repo-url> <image-repo>"
    exit 1
fi

GIT_REPO=$1
IMAGE_REPO=$2

gcloud builds submit "$GIT_REPO" \
    --git-source-revision=main --config=cloudbuild/build.yaml \
    --substitutions _IMAGE_URL="$IMAGE_REPO/frontend",_DIRECTORY="frontend"

gcloud builds submit "$GIT_REPO" \
    --git-source-revision=main --config=cloudbuild/build.yaml \
    --substitutions _IMAGE_URL="$IMAGE_REPO/backend",_DIRECTORY="backend"