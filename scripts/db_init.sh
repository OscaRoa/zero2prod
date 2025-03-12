#!/usr/bin/env bash
set -x
set -eo pipefail
source .env

# Check if a custom parameter has been set, otherwise use default values
DB_PORT="${POSTGRES_PORT}"

# Launch postgres using Docker
CONTAINER_NAME="postgres"

podman run \
  --env POSTGRES_USER="${SUPERUSER}" \
  --env POSTGRES_PASSWORD="${SUPERUSER_PWD}" \
  --health-cmd="pg_isready -U ${SUPERUSER} || exit 1" \
  --health-interval="1s" \
  --health-timeout="5s" \
  --health-retries=5 \
  --publish "${DB_PORT}":5432 \
  --detach \
  --name "${CONTAINER_NAME}" \
  --replace \
  postgres -N 1000
  
# Wait for Postgres to be ready to accept connections
until [ \
  "$(podman inspect -f "{{.State.Health.Status}}" ${CONTAINER_NAME})" == \
  "healthy" \
]; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"
  
# Create the application user
CREATE_QUERY="CREATE USER ${APP_USER} WITH PASSWORD '${APP_USER_PWD}';"
podman exec -it "${CONTAINER_NAME}" psql -U "${SUPERUSER}" -c "${CREATE_QUERY}"
# Grant create db privileges to the app user
GRANT_QUERY="ALTER USER ${APP_USER} CREATEDB;"
podman exec -it "${CONTAINER_NAME}" psql -U "${SUPERUSER}" -c "${GRANT_QUERY}"