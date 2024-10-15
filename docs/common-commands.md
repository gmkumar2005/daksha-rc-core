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