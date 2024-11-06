#!/usr/bin/env fish

# More info: https://diesel.rs/guides/getting-started.html

##############
# Constants. #
##############

set db_file "diesel.db"
set DATABASE_URL $db_file
set migration_name create_tables
set data_table_name data_table
set file_table_name file_table

#############################
# Define all the functions. #
#############################

function main

    # Check if migrations folder has already been created before. In this case
    # just run the migrations and exit.
    if test (check_if_migrations_folders_exist) = exists
        echo (set_color yellow)"migrations/*_$migration_name already exists, not re-creating them."(set_color normal)

        # Ask the user if they want to run the migrations.
        echo (set_color magenta)"Do you want to run existing migrations now? (y/n)"(set_color normal)
        read -P "> " -l run_existing_migrations
        if test $run_existing_migrations = y
            run_existing_migrations
            exit 0
        end

        # Ask the user if they want to create new migrations.
        echo (set_color magenta)"Do you want to generate new migrations now? (y/n)"(set_color normal)
        read -P "> " -l generate_new_migrations
        if test $generate_new_migrations = y
            generate_new_migrations
            exit 0
        end

        exit 0
    end

    generate_database
    generate_new_migrations

    echo (set_color magenta)"Now, populate the up.sql and down.sql files."(set_color normal)
    echo (set_color magenta)"No db file is created until you run the migrations."(set_color normal)
    echo (set_color magenta)"Once you've done that run the script again and this time run the migrations."(set_color normal)
end

function generate_database
    # Actually run the setup and migrations.
    echo (set_color green)"1. Create our database (if it didn’t already exist)."(set_color normal)
    echo (set_color green)"2. Set up the initial migrations directory, which will contain a generated migration file that establishes the Diesel setup."(set_color normal)
    echo (set_color green)"Note: the migrations directory will not be empty as the initial setup migration is automatically generated."(set_color normal)
    diesel setup
end

function generate_new_migrations
    echo (set_color green)"Diesel CLI will create two empty files for us: up.sql and down.sql."(set_color normal)
    echo (set_color green)"You will see this in the <timestamp>_$migration_name folder. Write SQL to create and delete $data_table_name and $file_table_name tables."(set_color normal)
    diesel migration generate $migration_name
end

function run_existing_migrations
    echo (set_color green)"Running migrations & creating schema.rs..."(set_color normal)
    # This will exercise the `up.sql` file.
    diesel migration run --database-url=diesel.db
    # This will exercise the `down.sql`, then `up.sql` file.
    diesel migration redo --database-url=diesel.db
end

function check_if_migrations_folders_exist
    set -l retval does_not_exist
    if test -d migrations
        if test -d migrations/*_$migration_name
            set retval exists
        end
    end
    echo $retval
end

function install_libsqlite3
    set -l packageIsInstalled (dpkg -l "libsqlite3-dev")
    if test -z "$packageIsInstalled"
        # Non zero exit code was returned in $status.
        echo (set_color yellow)"libsqlite3 not found, installing it now..."(set_color normal)
        sudo apt-get install libsqlite3-dev
    else
        echo (set_color yellow)"libsqlite3 already installed."(set_color normal)
    end
end

function install_diesel_cli
    set -l output (cargo install --list | rg diesel_cli)
    # If $output starts with "diesel_cli" then we have it installed.
    if not test (string match -r "^diesel_cli" $output)
        echo (set_color yellow)"Diesel CLI not found, installing it now..."(set_color normal)
        cargo install diesel_cli --no-default-features --features sqlite
    else
        echo (set_color yellow)"Diesel CLI already installed."(set_color normal)
    end
end

############################
# Actually run the script. #
############################
install_libsqlite3
install_diesel_cli
main
