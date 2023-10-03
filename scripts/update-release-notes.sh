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

RELEASE_NOTES="${REPO_ROOT}/RELEASE_NOTES.md"
/usr/bin/cat "${REPO_ROOT}/docs/release-notes-template.md" | tee "${RELEASE_NOTES}"

for note in $(/usr/bin/find "${REPO_ROOT}/docs" -name "changelog*.md" -print | sort -rn); do
  /usr/bin/cat "${note}" | tee -a "${RELEASE_NOTES}"
  echo "" | tee -a "${RELEASE_NOTES}"
  echo "" | tee -a "${RELEASE_NOTES}"
done
