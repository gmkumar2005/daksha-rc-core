### Use podman compose to start database

```shell
podman compose up -d
```

### Enable docker host access via podman

```shell
export DOCKER_HOST="unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')"
```

### Copy log files from container to host

```shell
podman cp sunbird-rc-core-registry-1:/app/logs/app.log app.log

```

### Run sqlx migrations

```shell

env $(envsubst < .env | xargs) cargo sqlx prepare
env $(envsubst < .env | xargs) cargo sqlx migrate info --source ../migrations
env $(envsubst < .env | xargs) cargo sqlx migrate run --source ../migrations
```

### Helm commands

``
helm show values runix/pgadmin4 > pgadminvalues.yaml
helm template dev-app rc-app > demo.yaml
helm install <release-name> . --dry-run --debug
helm install my-postgresql bitnami/postgresql --dry-run --debug -f pgvalues.yaml > demo.yaml

helm template dev-app bitnami/postgresql -f pgvalues.yaml > demo.yaml

helm show values bitnami/postgresql > pgvalues.yaml

kubectl get secret my-postgresql -o yaml

podman exec -it kind-control-plane crictl images
kind load docker-image my-image:tag

kind load docker-image docker.io/library/rc-web:latest

KIND_EXPERIMENTAL_PROVIDER=podman kind create cluster
kubectl cluster-info

kubectl rollout status deployment \
-n cnpg-system cnpg-controller-manager
``

## Installing postgres

```shell
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm upgrade --install cnpg \
  --namespace cnpg-system \
  --create-namespace \
  cnpg/cloudnative-pg


```

## Manual deploy commands

```shell
KIND_EXPERIMENTAL_PROVIDER=podman kind create cluster
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm upgrade --install cnpg \
  --namespace cnpg-system \
  --create-namespace \
  cnpg/cloudnative-pg

kubectl get deployment -n cnpg-system

kubectl apply -f k8s/manual/pgcluster.yaml
kubectl get cluster rc-database
// build app container image
kind load docker-image docker.io/library/rc-web:latest
kubectl apply -f k8s/manual/rc-web-deployment.yaml
kubectl rollout status deployment/rc-web
kubectl port-forward service/rc-web 8000:8000 &

```

## Helm based install

```shell
cd k8s
KIND_EXPERIMENTAL_PROVIDER=podman kind create cluster
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm upgrade --install cnpg \
  --namespace cnpg-system \
  --create-namespace \
  cnpg/cloudnative-pg


kubectl wait --for=condition=Available deployment/cnpg-cloudnative-pg -n cnpg-system --timeout=120s

// build app container image
kind load docker-image docker.io/library/rc-web:latest
helm install dev rc-app
helm status dev
kubectl wait --for=condition=Available deployment/dev-rc-app -n default --timeout=120s

curl -k https://rc.127.0.0.1.nip.io/healthz

kubectl port-forward service/dev-rc-app 8000:8000 &

curl http://localhost:50000/healthz

```

## Mirrord commands

```shell

mirrord exec cargo run  --target pod/dev-rc-app-ffc4969db-4zjcv

RUST_LOG=debug mirrord exec cargo run --target pod/dev-rc-app-ffc4969db-4zjcv

RUST_LOG=rc_web=debug mirrord exec cargo run --target pod/dev-rc-app-ffc4969db-4zjcv
```

## Tag container images

```shell

echo github_pat | docker login ghcr.io -u gmkumar2005 --password-stdin
ghcr.io -u gmkumar2005 --password-stdin

docker tag docker.io/library/rc-web  ghcr.io/daksha-rc/rc-web:0.0.10

docker push  ghcr.io/daksha-rc/rc-web:0.0.10


```

## Trafeak

```shell
KIND_EXPERIMENTAL_PROVIDER=podman kind create cluster --config kind-config.yaml

kubectl label node kind-control-plane ingress-ready=true

helm repo add traefik https://helm.traefik.io/traefik
helm repo update

helm install traefik-crds traefik/traefik-crds
<!-- helm install traefik traefik/traefik --set crds.enabled=true -->
helm upgrade --install traefik traefik/traefik -f k8s/traefik-values.yaml
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=traefik --timeout=60s

kubectl create namespace httpbin
kubectl apply -n httpbin -f https://github.com/istio/istio/raw/master/samples/httpbin/httpbin.yaml

kubectl apply -f whoami.yaml
kubectl apply -f whoami-service.yaml

kubectl apply -f httpbin-ingressroute.yaml
kubectl apply -f whoami-ingressroute.yaml
kubectl apply -f traefik-dashboard-ingressroute.yaml

curl -k https://httpbin.127.0.0.1.nip.io/get
curl -k https://whoami.127.0.0.1.nip.io/

```

## Debiug

```shell
kubectl get pods -n default -o wide | grep traefik

```

## Install treafik and cnpg with KIND

```shell
KIND_EXPERIMENTAL_PROVIDER=podman kind create cluster --config kind-config.yaml
helm repo add traefik https://helm.traefik.io/traefik
helm repo update
helm install traefik-crds traefik/traefik-crds
helm upgrade --install traefik traefik/traefik -f k8s/traefik-values.yaml
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=traefik --timeout=60s
kubectl apply -f k8s/manual/traefik-dashboard-ingressroute.yaml


kubectl apply -f k8s/manual/whoami.yaml
kubectl apply -n httpbin -f k8s/manual/httpbin.yaml

curl -k https://httpbin.127.0.0.1.nip.io/get
curl -k https://whoami.127.0.0.1.nip.io/

```

## Install cnpg

```shell
helm upgrade --install cnpg \
  --namespace cnpg-system \
  --create-namespace \
  cnpg/cloudnative-pg

kubectl wait --for=condition=Available deployment/cnpg-cloudnative-pg -n cnpg-system --timeout=120s

helm install dev rc-app
kubectl wait --for=condition=Available deployment/dev-rc-app -n default --timeout=120s
curl -k https://rc.127.0.0.1.nip.io/healthz

```

### Connecting to postgres on k8s

```shell

Service name dev-rc-app-database-rw
Secrets name dev-rc-app-database-app
kubectl get secret dev-rc-app-database-app -n default -o jsonpath="{.data.username}" | base64 --decode && echo
kubectl get secret dev-rc-app-database-app -n default -o jsonpath="{.data.password}" | base64 --decode && echo
kubectl get secret dev-rc-app-database-app -n default -o jsonpath="{.data.dbname}" | base64 --decode && echo
kubectl port-forward svc/dev-rc-app-database-app -n default 5432:5432

PGPASSWORD=<password> psql -h localhost -p 5432 -U <username> -d <dbname>


```

###

```shell
psql postgresql://daksha_rc:NYpVusIWuctlwdOh60SroevN2BFizyw4YomTKMXHZo4gAn8ou0uN5aF4lwB8IgWk@localhost:5432/daksha_rc

```

### Pull image debug

```shell

ghcr.io/daksha-rc/rc-web:v.0.1.1
ghcr.io/daksha-rc/rc-web:v0.1.1
docker pull ghcr.io/daksha-rc/rc-web:v0.1.1
```

### Sit cluster

#### Debug

```shell
kubectl --kubeconfig=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml config current-context
kubectl --kubeconfig=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml get svc/nginx-service
```

#### Install

```shell
export KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml
kubectl --kubeconfig=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml config current-context
helm --kubeconfig=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml  install traefik-crds traefik/traefik-crds --namespace traefik-system --create-namespace

helm --kubeconfig=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml  upgrade --install traefik traefik/traefik -f k8s/manual/sit-traefik-values.yaml --namespace traefik-system --create-namespace

kubectl --kubeconfig=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml wait --for=condition=ready pod -l app.kubernetes.io/name=traefik -n traefik-system --timeout=60s

kubectl --kubeconfig=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml apply -f k8s/manual/sit-traefik-dashboard-ingressroute.yaml

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml helm upgrade --install cnpg \
  --namespace cnpg-system \
  --create-namespace \
  cnpg/cloudnative-pg

kubectl wait --for=condition=Available deployment/cnpg-cloudnative-pg -n cnpg-system --timeout=120s

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml helm upgrade --install sit rc-app -f manual/sit-rc-app-values.yaml
KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl logs
KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml k9s --readonly

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml  helm uninstall cnpg \
  --namespace cnpg-system \
  cnpg/cloudnative-pg

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml  helm uninstall  sit rc-app
KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl delete metrics-server-5b6cc55b86-j752f -n kube-system

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl delete metrics-server-5cd4986bbc-6sqd9 -n kube-system
```

## Multi platform builds

```shell
podman build -f rc-web/Dockerfile --manifest ghcr.io/daksha-rc/rc-web:v0.1.1-dev.4 .

podman build -f rc-web/Dockerfile --platform linux/amd64,linux/arm64  --manifest ghcr.io/daksha-rc/rc-web:v0.1.1-dev.4 .

podman build --platform linux/amd64,linux/arm64 --manifest localhost/hello .

podman manifest inspect ghcr.io/daksha-rc/rc-web:v0.1.5 | jq -r '.manifests[].platform | "\(.os)/\(.architecture)"'


podman image inspect ghcr.io/daksha-rc/rc-web:v0.1.1-dev.4 --format '{{json .Architecture}}'

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl get nodes -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.nodeInfo.architecture}{"\t"}{.status.nodeInfo.operatingSystem}{"\n"}{end}'

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl patch storageclass utho-block-storage -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl patch pvc sit-rc-app-database-1 -p '{"spec": {"storageClassName": "utho-block-storage"}}'

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl get storageclass
```

## Multi platform build

```shell

podman build --arch amd64 -t ghcr.io/daksha-rc/rc-web:amd64 .
podman build --arch arm64 -t ghcr.io/daksha-rc/rc-web:arm64 .
podman manifest create ghcr.io/daksha-rc/rc-web:latest
podman manifest add ghcr.io/daksha-rc/rc-web:latest containers-storage:ghcr.io/daksha-rc/rc-web:amd64
podman manifest add ghcr.io/daksha-rc/rc-web:latest containers-storage:ghcr.io/daksha-rc/rc-web:arm64
podman manifest inspect ghcr.io/daksha-rc/rc-web:latest

```

## Debug remote k8s

```shell
KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl get nodes --show-labels

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl cluster-info
KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl get cluster
KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl get nodes -o wide

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml kubectl get pod sit-rc-app-5c9fcd44d5-2s72w -o jsonpath='{.spec.containers[0].image}'

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml  kubectl get pvc -l cnpg.io/cluster=sit-rc-app-database  -n default

KUBECONFIG=/Users/mallru/.kube/SIT-Daksha-kubeconfig_mks_750390.yaml  kubectl get storageclass


TAG=v0.1.9 cargo make build-image-all
TAG=v0.1.9 cargo make push-with-tag


```

# Start of digital ocean deployemnt

```shell
export KUBECONFIG=/Users/mallru/.kube/sit-daksha-kubeconfig.yaml
helm repo update
helm install sit-db bitnami/postgresql -f do-sit-pg-values.yaml
sit-db-postgresql.default.svc.cluster.local - Read/Write connection
export POSTGRES_ADMIN_PASSWORD=$(kubectl get secret --namespace default sit-db-postgresql -o jsonpath="{.data.postgres-password}" | base64 -d)
export POSTGRES_PASSWORD=$(kubectl get secret --namespace default sit-db-postgresql -o jsonpath="{.data.password}" | base64 -d)
kubectl run sit-db-postgresql-client --rm --tty -i --restart='Never' --namespace default --image docker.io/bitnami/postgresql:17.5.0-debian-12-r12 --env="PGPASSWORD=$POSTGRES_PASSWORD" \
      --command -- psql --host sit-db-postgresql -U daksha_rc -d daksha_rc -p 5432

postgresql://user:secretpassword@localhost:5432/mydatabase
postgresql://daksha_rc:daksha_rc@sit-db-postgresql.default.svc.cluster.local:5432/daksha_rc

```

## Treafik installation

```shell
helm install traefik-crds traefik/traefik-crds

helm upgrade --install traefik traefik/traefik -f do-sit-traefik-values.yaml
#traefik with docker.io/traefik:v3.4.1 has been deployed successfully on default namespace !
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=traefik --timeout=60s
kubectl apply -f do-sit-dashboard-ingressroute.yaml

helm uninstall traefik traefik/traefik
http://dashboard.68.183.244.95.nip.io/dashboard/#/

```

## install rc-app on do

```shell

helm upgrade --install sit rc-app -f manual/do-sit-rc-values.yaml --dry-run --debug
helm upgrade --install  sit rc-app -f manual/do-sit-rc-values.yaml
helm uninstall  dev
kubectl apply -f do-sit-rc-ingressroute.yaml
```

## DO Dns

```shell
dig rc.daksha-rc.in @8.8.8.8 +short
dig dashboard.daksha-rc.in @8.8.8.8 +short
dig rc.daksha-rc.in  +short

doctl compute certificate create --type lets_encrypt --name daksha-rc-subdomains-cert --dns-names rc.daksha-rc.in,dashboard.daksha-rc.in


kubectl patch service traefik -n default --type='merge' -p '{
  "metadata": {
    "annotations": {
      "service.beta.kubernetes.io/do-loadbalancer-protocol": "https",
      "service.beta.kubernetes.io/do-loadbalancer-certificate-name": "daksha-rc-subdomains-cert",
      "service.beta.kubernetes.io/do-loadbalancer-certificate-id": "806c5cb4-7f4f-4147-b962-7b583f9e4b68",
      "service.beta.kubernetes.io/do-loadbalancer-disable-lets-encrypt-dns-records": "false",
      "service.beta.kubernetes.io/do-loadbalancer-redirect-http-to-https": "true"
    }
  }
}'


doctl compute load-balancer update 7edd6f38-ca2a-4a2c-a3d0-87cc28697eb7 \
  --forwarding-rules \
    entry_protocol:http,entry_port:80,target_protocol:http,target_port:80 \
    entry_protocol:tcp,entry_port:8080,target_protocol:tcp,target_port:8080 \
    entry_protocol:https,entry_port:443,target_protocol:http,target_port:80,certificate_id:806c5cb4-7f4f-4147-b962-7b583f9e4b68




```

```shell

protocol:http,port:10256,path:/healthz,check_interval_seconds:3,response_timeout_seconds:5,healthy_threshold:5,unhealthy_threshold:3,proxy_protocol:0xc0002576f0
entry_protocol:http,entry_port:8080,target_protocol:http,target_port:30090,certificate_id:,tls_passthrough:false
entry_protocol:http,entry_port:80,target_protocol:http,target_port:30080,certificate_id:,tls_passthrough:false
entry_protocol:https,entry_port:443,target_protocol:http,target_port:30443,certificate_id:806c5cb4-7f4f-4147-b962-7b583f9e4b68,tls_passthrough:false


entry_protocol:http,entry_port:8080,target_protocol:http,target_port:30090,certificate_id:,tls_passthrough:false
entry_protocol:http,entry_port:80,target_protocol:http,target_port:30080,certificate_id:,tls_passthrough:false
entry_protocol:https,entry_port:443,target_protocol:http,target_port:30443,certificate_id:806c5cb4-7f4f-4147-b962-7b583f9e4b68,tls_passthrough:false

entry_protocol:tcp,entry_port:8080,target_protocol:tcp,target_port:8080,certificate_id:,tls_passthrough:false entry_protocol:tcp,entry_port:80,target_protocol:tcp,target_port:80,certificate_id:,tls_passthrough:false

entry_protocol:tcp,entry_port:8080,target_protocol:tcp,target_port:8080,certificate_id:,tls_passthrough:false entry_protocol:tcp,entry_port:80,target_protocol:tcp,target_port:80,certificate_id:,tls_passthrough:false entry_protocol:tcp,entry_port:443,target_protocol:tcp,target_port:443,certificate_id:,tls_passthrough:false

entry_protocol:tcp,entry_port:8080,target_protocol:tcp,target_port:8080,certificate_id:,tls_passthrough:false entry_protocol:tcp,entry_port:80,target_protocol:tcp,target_port:80,certificate_id:,tls_passthrough:false entry_protocol:tcp,entry_port:443,target_protocol:tcp,target_port:443,certificate_id:,tls_passthrough:false


```
