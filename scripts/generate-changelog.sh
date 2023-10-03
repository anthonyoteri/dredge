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

