#!/bin/bash
#set -xv

# The UI's login link is built for this portal url (it must be an allowed redirect of the
# tms client).  Override it for other hosts, e.g. REDIRECT_URI=https://<portal host>/
REDIRECT_URI="${REDIRECT_URI:-http://localhost:8080/}"

# installs the pnpm version pinned by packageManager in client/package.json
docker run -it -e REDIRECT_URI="$REDIRECT_URI" -v `pwd`/client:`pwd`/client -w `pwd`/client node:24 \
  /bin/bash -c 'npm install -g "$(node -p "require(\"./package.json\").packageManager")" && pnpm install && pnpm run build'
