#!/bin/bash
if command -v kubectl >/dev/null 2>&1; then echo "kubectl already installed"; exit 0; fi
KUBECTL_VERSION=v1.30.2
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
  curl -LO https://dl.k8s.io/release/$KUBECTL_VERSION/bin/darwin/arm64/kubectl
else
  curl -LO https://dl.k8s.io/release/$KUBECTL_VERSION/bin/darwin/amd64/kubectl
fi
chmod +x kubectl
sudo mv kubectl /usr/local/bin/