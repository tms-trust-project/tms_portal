#!/bin/bash

set -e

docker build -f services/tms_portal/Dockerfile -t tapis/tms-api:local --build-arg SERVICE_NAME=tms_portal .

minikube image load tapis/tms-api:local --overwrite
