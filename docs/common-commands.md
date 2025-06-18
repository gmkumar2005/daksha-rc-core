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
