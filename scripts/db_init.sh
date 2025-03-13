#!/usr/bin/env bash
set -x
set -eo pipefail
source .env

# Ensure podman-compose is installed
if ! [ -x "$(command -v podman-compose)" ]; then
  echo >&2 "Error: podman-compose is not installed."
  exit 1
fi

# Ensure sqlx is installed
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 " cargo install sqlx-cli --no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

# Start the db container if it's not already running
podman-compose up --no-recreate -d
podman-compose run -d web

# Wait for the db container to be ready
until podman-compose exec db pg_isready -U postgres; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

# Create the user and grant permissions
CREATE_USER_QUERY=$(cat <<EOF
DO \$$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = '${APP_USER}') THEN
    CREATE USER ${APP_USER} WITH PASSWORD '${APP_USER_PWD}';
  END IF;
END
\$$;
EOF
)

# Grant CREATEDB permission if it hasn't already been granted
GRANT_CREATEDB_QUERY=$(cat <<EOF
DO \$$
BEGIN
  IF NOT EXISTS (
    SELECT FROM pg_catalog.pg_roles
    WHERE rolname = '${APP_USER}' AND rolcreatedb = TRUE
  ) THEN
    ALTER USER ${APP_USER} CREATEDB;
  END IF;
END
\$$;
EOF
)
podman-compose exec db psql -U postgres -c "${CREATE_USER_QUERY}"
podman-compose exec db psql -U postgres -c "${GRANT_CREATEDB_QUERY}"

# Set the DATABASE_URL to point to the db container
DATABASE_URL=postgres://${APP_USER}:${APP_USER_PWD}@localhost:5433/${APP_DB_NAME}
export DATABASE_URL

# Create the database and run migrations
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"