#!/bin/bash

PACKAGE_VERSION=$(cat package.json | jq -r '.version')

PACKAGE_NAME=$(cat package.json | jq -r '.name')

echo $PACKAGE_NAME

FOUND_VERSION=$(yarn npm info "$PACKAGE_NAME" -f versions | grep "$PACKAGE_VERSION" | xargs)

# If found version is unset or the empty string we need to publish otherwise echo that there is already a published version
if [ -z "${FOUND_VERSION}" ]; then
  moon $PACKAGE_NAME:build && yarn npm publish --access public
else
  echo "Will not publish package $PACKAGE_NAME, as the version in package.json is already published"
  echo "Found version: $FOUND_VERSION, Package Version: $PACKAGE_VERSION"
fi