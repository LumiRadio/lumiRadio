#!/bin/sh

# Abort on any error (including if wait-for-it fails).
set -e

# Wait for the backend to be up, if we know where it is.
if [ -n "$LIQUIDSOAP_HOST" ] || [ -n "$LIQUIDSOAP_PORT" ]; then
    /usr/local/bin/wait-for-it.sh "$LIQUIDSOAP_HOST:$LIQUIDSOAP_PORT" -s --timeout=60
fi

# Run the main container command.
exec "$@"