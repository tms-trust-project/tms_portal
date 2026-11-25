#!/bin/bash

kubectl apply -f config-map.yml
kubectl apply -f service.yml
kubectl apply -f deploy.yml

