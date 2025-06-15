#!/bin/bash
if command -v kubectl >/dev/null 2>&1; then echo "kubectl already installed"; exit 0; fi
KUBECTL_VERSION=v1.30.2
curl -LO https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/amd64/kubectl
chmod +x kubectl
sudo mv kubectl /usr/local/bin/