
### Run sqlx migrations

```shell

env $(envsubst < .env | xargs) cargo sqlx prepare
env $(envsubst < .env | xargs) cargo sqlx migrate info --source ../migrations
env $(envsubst < .env | xargs) cargo sqlx migrate run --source ../migrations
```


## Mirrord commands

```shell

mirrord exec cargo run  --target pod/dev-rc-app-ffc4969db-4zjcv

RUST_LOG=debug mirrord exec cargo run --target pod/dev-rc-app-ffc4969db-4zjcv

RUST_LOG=rc_web=debug mirrord exec cargo run --target pod/dev-rc-app-ffc4969db-4zjcv
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


### Sit cluster


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






```

```shell
Tokens

DAKSHA_RC_ACTIONS_WRITE_PAT
```

## Lets encrypt

```shell

export DIGITALOCEAN_ACCESS_TOKEN="your_digitalocean_api_token"
export DO_AUTH_TOKEN="your_digitalocean_api_token_here"


lego --dns digitalocean --domains daksha-rc.in --domains '*.daksha-rc.in' --email dmkumar2014@gmail.com run

lego --dns digitalocean  --dns-timeout 1800 --dns.propagation-wait 1800s   --domains daksha-rc.in --domains '*.daksha-rc.in' --email dmkumar2014@gmail.com run
  
  run


kubectl create secret tls daksha-rc-tls \
  --cert=daksha-rc.in.crt \
  --key=daksha-rc.in.key 

```

```shell

build 

docker pull ghcr.io/daksha-rc/rc-web:v0.1.3


nerdctl build \
  --platform=linux/amd64,linux/arm64 \
  --output type=image,name=ghcr.io/daksha-rc/rc-web:latest,push=true \
  .

nerdctl build \
  --platform=linux/amd64,linux/arm64 \
  --output type=image,name=ghcr.io/daksha-rc/rc-web:latest,push=true \
  -f rc-web/Dockerfile \
  .



nerdctl build \
  --platform=linux/amd64,linux/arm64 \
  --output type=image,name=ghcr.io/daksha-rc/rc-web:latest,push=false \
  -f rc-web/Dockerfile \
  .

nerdctl build \
  --platform=linux/amd64,linux/arm64 \
  --output type=image,name=ghcr.io/daksha-rc/rc-web:latest,oci-mediatypes=true,oci-store=true \
  -f rc-web/Dockerfile \
  .


nerdctl push ghcr.io/daksha-rc/rc-web:latest

docker pull ghcr.io/daksha-rc/rc-web:v0.1.3

```

```shell
nerdctl pull alpine
nerdctl tag alpine ghcr.io/daksha-rc/rc-web:latest
nerdctl push ghcr.io/daksha-rc/rc-web:latest

nerdctl image prune -a -f


nerdctl --namespace buildkit image prune -a -f

nerdctl --namespace buildkit pull --platform linux/arm64 ghcr.io/daksha-rc/rc-web:latest 
nerdctl --namespace buildkit pull --platform linux/amd64 ghcr.io/daksha-rc/rc-web:latest 
nerdctl --namespace buildkit images

```

```shell

nerdctl run --rm --platform=amd64 alpine uname -a

nerdctl build --platform=linux/amd64,linux/arm64 -t ghcr.io/daksha-rc/rc-web:latest -f rc-web/Dockerfile .

nerdctl build  -t ghcr.io/daksha-rc/rc-web:latest -f rc-web/Dockerfile .
  
nerdctl push --all-platforms ghcr.io/daksha-rc/rc-web:latest

```

```shell

echo $(cat /Users/mallru/.ssh/PUSH_PKG_TO_RC_ORG.token) | nerdctl login ghcr.io -u gmkumar2005 --password-stdin
```

```shell
release-please release-pr \
--repo-url=https://github.com/gmkumar2005/daksha-rc-core \
--package-name=rc-web \
--release-type=simple \
--token="$GITHUB_TOKEN" \
--target-branch=53_caching \
--dry-run

```