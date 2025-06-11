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

KIND_EXPERIMENTAL_PROVIDER=podman kind create cluster
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm upgrade --install cnpg \
  --namespace cnpg-system \
  --create-namespace \
  cnpg/cloudnative-pg

// build app container image 
kind load docker-image docker.io/library/rc-web:latest  
helm install dev rc-app
helm status dev
 
kubectl port-forward service/dev-rc-app 8000:8000 &

curl http://localhost:50000/healthz

```