#!/bin/bash

sudo systemctl start postgresql

echo "fix start_db.sh"
exit 1

#USER="postgres"
#DB_NAME="flexforum"
source ./backend/database/.env

if ! PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -lqt >/dev/null 2>&1; then
    echo "Failed to connect to database. Check credentials."
    exit 1
fi

if ! PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    echo "Creating database '$DB_NAME'..."
    createdb -U "$DB_USER" -h "$DB_HOST" -p "$DB_PORT" "$DB_NAME"
    echo "Database '$DB_NAME' created."
fi

#DIR="/var/lib/postgres/data"
# USER="postgres"
# DB_NAME="flexforum"

# # Check if database does NOT exist - create it
# if ! sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
#     # Create the database
#     echo "Creating database '$DB_NAME'..."
#     createdb -U "$USER" "$DB_NAME"
#     echo "Database '$DB_NAME' created."
# fi

#sudo -u postgres pg_ctl start -D $DIR
