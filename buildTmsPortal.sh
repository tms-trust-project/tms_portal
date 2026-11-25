#!/bin/bash

set -e

# the web client is built into the image for this redirect uri (override for other hosts)
REDIRECT_URI="${REDIRECT_URI:-http://localhost:8080/}"

docker build -f services/tms-api/Dockerfile -t tapis/tms-api:local \
  --build-arg SERVICE_NAME=tms-api --build-arg REDIRECT_URI="$REDIRECT_URI" .

# load the image into minikube for the local k8s deployment (deploy/k8s/local); without
# minikube the image is only built, e.g. to run it with docker
if command -v minikube >/dev/null 2>&1; then
  minikube image load tapis/tms-api:local --overwrite
else
  echo "minikube isn't installed: built tapis/tms-api:local, but didn't load it into minikube"
fi
