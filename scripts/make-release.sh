#
#    Copyright 2023 Anthony Oteri
#
#    Licensed under the Apache License, Version 2.0 (the "License");
#    you may not use this file except in compliance with the License.
#    You may obtain a copy of the License at
#
#        http://www.apache.org/licenses/LICENSE-2.0
#
#    Unless required by applicable law or agreed to in writing, software
#    distributed under the License is distributed on an "AS IS" BASIS,
#    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#    See the License for the specific language governing permissions and
#    limitations under the License.
#

#!/usr/bin/bash -e

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


