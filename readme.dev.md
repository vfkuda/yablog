# required packages/crates 
```
# rustup target add wasm32-unknown-unknown
# brew install wasm-pack
# brew install just
```

# building apps
##  build all project crates
```
cargo build --workspace --all-targets
```

## creating frontend app (web pack)
```
wasm-pack build blog-wasm --target web
```

# running
## running frontend application (PYTHON 3.x is required)
```
cd blog-wasm; python3 -m http.server 8081
```

# Use docker images
## running database 
```bash
docker run --rm --name yablog-postgres \
  -e POSTGRES_USER=user \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=blog_db \
  -p 5432:5432 \
  -d postgres:16-alpine
```

## backend docker image
* build yablog server (backend) image
```
docker build -f Dockerfile.blog-server -t yablog-server:local .
```
* and run container
```
docker run --name yablog-server \
  -e DATABASE_URL=postgres://user:password@host.docker.internal:5432/blog_db \
  -e JWT_SECRET=change-me-please-minimum-32-characters-secret-key \
  -e APP_ENV=production \
  -e CORS_ALLOWED_ORIGINS=http://localhost:8081 \
  -p 8080:8080 \
  -p 50051:50051 \
  -d yablog-server:local
```

## frontend docker image
* build yablog  (frontend) image
```bash
docker build -f Dockerfile.blog-wasm -t yablog-wasm:local .
```
* and run container
```
docker run --name yablog-wasm \
  -p 8081:80 \
  -d yablog-wasm:local
```

## stopping containers:
```bash
docker rm -f yablog-wasm yablog-server yablog-postgres
```

## quick start/stop
* start
```bash
./build.sh
./start.sh
```
* test
```
# frontend: `http://localhost:8081`
# backend HTTP: `http://localhost:8080`
# backend gRPC: `localhost:50051`
# postgres: `localhost:5432`
```
* stops
```bash
./stop.sh
```


