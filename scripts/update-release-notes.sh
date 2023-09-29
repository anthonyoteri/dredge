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

REPO_ROOT=$(git rev-parse --show-toplevel)

/usr/bin/cat "${REPO_ROOT}/docs/release-notes-template.md" $(/usr/bin/ls "${REPO_ROOT}/docs/changelog*.md" 2>/dev/null | true | sort -rn) | tee "${REPO_ROOT}/RELEASE_NOTES.md"