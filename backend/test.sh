#!/usr/bin/env bash
set -e -o pipefail

# Start PostgreSQL container
docker run --rm -d --replace \
  --name test-db \
  -e POSTGRES_USER=test \
  -e POSTGRES_PASSWORD=test \
  -e POSTGRES_DB=test \
  -p 5432:5432 \
  docker.io/postgres:17-alpine

# Wait for the DB to be ready
until docker exec test-db pg_isready -U test >/dev/null 2>&1; do
  echo "Waiting for PostgreSQL..."
  sleep 1
done

# Set env var and run tests
export DATABASE_URL=postgres://test:test@localhost:5432/test

cargo test "$@"