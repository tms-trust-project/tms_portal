#!/bin/bash

set -e

here=$(pwd)

# Create the secrets
kubectl apply -f secrets.yml

# Start tms db
#cd "$here/postgres/"
#./burnup.sh
#kubectl rollout status deployment/tms-postgres --timeout=5m

# Start tms portal
cd "$here/tms-portal/"
./burnup.sh
kubectl rollout status deployment/tms-portal --timeout=5m
