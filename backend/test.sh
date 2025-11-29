set -e -o pipefail

docker run --rm -d --replace \
    --name chronicle-test-db \
    -e POSTGRES_USER=chronicle \
    -e POSTGRES_PASSWORD=password \
    -e POSTGRES_DB=chronicle \
    -p 5432:5432 \
    docker.io/postgres:17-alpine

until docker exec chronicle-test-db pg_isready -U chronicle; do
  sleep 1
done

export DATABASE_URL=postgres://chronicle:password@localhost:5432/chronicle

cargo test "$@"