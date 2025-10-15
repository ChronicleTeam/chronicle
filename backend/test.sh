set -e -o pipefail

docker run --rm -d --replace \
  --name test-db \
  -e POSTGRES_USER=test \
  -e POSTGRES_PASSWORD=test \
  -e POSTGRES_DB=test \
  -p 5432:5432 \
  docker.io/postgres:17-alpine

until docker exec test-db pg_isready -U test >/dev/null 2>&1; do
  echo "Waiting for PostgreSQL..."
  sleep 1
done

export DATABASE_URL=postgres://test:test@localhost:5432/test

cargo test "$@"