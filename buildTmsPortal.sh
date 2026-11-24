#!/bin/bash

set -e

# the web client is built into the image for this redirect uri (override for other hosts)
REDIRECT_URI="${REDIRECT_URI:-http://localhost:8080/}"

docker build -f services/tms-api/Dockerfile -t tapis/tms-api:local \
  --build-arg SERVICE_NAME=tms-api --build-arg REDIRECT_URI="$REDIRECT_URI" .

minikube image load tapis/tms-api:local --overwrite
