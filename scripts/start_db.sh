#!/bin/bash

sudo systemctl start postgresql

#DIR="/var/lib/postgres/data"
USER="postgres"
DB_NAME="flexforum"

# Check if database does NOT exist - create it
if ! sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    # Create the database
    echo "Creating database '$DB_NAME'..."
    createdb -U "$USER" "$DB_NAME"
    echo "Database '$DB_NAME' created."
fi

#sudo -u postgres pg_ctl start -D $DIR
