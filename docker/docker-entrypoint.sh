#!/bin/sh

set -e

if [ -n "$DATABASE_URL" ]; then
    /usr/local/bin/wait-for-it.sh "db:5432" -s --timeout=60 -- echo "Postgres is up"
fi

exec "$@"