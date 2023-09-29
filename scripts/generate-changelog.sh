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

#!/usr/bin/bash

set -e

REPO_ROOT=$(git rev-parse --show-toplevel)

version=$1

if [ -z "$1" ]; then
  echo "Usage $0 <version> [from tag]"
  exit 1
fi

previous=${2:-$(git describe --abbrev=0 --match='v*')}

changelog="${REPO_ROOT}/docs/changelog-${previous}-${version}.md"

echo "- ${version}" | tee ${changelog}
echo "" | tee -a ${changelog}
git log --pretty=format:'  - %s by %an %h' --no-merges ${previous}.. | tee -a ${changelog}

