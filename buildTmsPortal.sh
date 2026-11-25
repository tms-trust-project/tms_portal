#!/bin/bash

set -e

docker build -f services/tms-api/Dockerfile -t tapis/tms-api:local --build-arg SERVICE_NAME=tms-api .

minikube image load tapis/tms-api:local --overwrite
