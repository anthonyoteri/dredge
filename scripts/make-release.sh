#!/usr/bin/bash

#
# Copyright 2023 Anthony Oteri
#
# Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
# http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
# http://opensource.org/licenses/MIT>, at your option. This file may not be
# copied, modified, or distributed except according to those terms.
#

set -e

version=$1
previous=$2

if [ -z "$1" ]; then
  echo "Usage $0 <version> [previous]"
  exit 1
fi

REPO_ROOT=$(git rev-parse --show-toplevel)
SCRIPTS="${REPO_ROOT}/scripts"

${SCRIPTS}/generate-changelog.sh "v${version}" "${previous}" && \
  ${SCRIPTS}/update-release-notes.sh && \
  git add "${REPO_ROOT}/docs" "${REPO_ROOT}/RELEASE_NOTES.md"

sed -i "s/^version = \".*\"/version = \"${version}\"/" \
    "${REPO_ROOT}/Cargo.toml" && git add "${REPO_ROOT}/Cargo.toml"

echo "*************************************************************************"
echo "Release is ready, please use the following to commit changes"
echo
echo "git commit -am \"Release version ${version}\" && \ "
echo "   git tag -a -m v${version} v${version}"
echo
echo "*************************************************************************"


