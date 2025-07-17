# Tools

docker compose down

```bash
docker compose -f tools/docker-compose/databases.yaml down -v --remove-orphans 
```

docker compose up

```bash
docker compose -f tools/docker-compose/databases.yaml up --build -d
```
