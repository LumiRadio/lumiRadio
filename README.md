# lumiRadio

The Homestuck Radio, now powered by Liquidsoap and a bunch of Rust code.

## Development

In order to develop and test for the radio, you can follow the steps below to get a local development version running. Keep in mind that you will need to have Docker and Git installed on your system.

Another dependency that the radio requires is `ffmpeg` and the headers for it as well as `Clang`.

Additionally, as this project uses sqlx, you will need to install the sqlx-cli (`cargo install sqlx-cli`).

1. You will need to [create a Discord bot](https://discord.com/developers/applications)
    - Make note of your client ID, client secret and the bot token, you will need those later!
2. Prepare some music and make note of the path to it
    - This doesn't have to be Homestuck music but can be any music which has some general ID3v2 tags on them
3. Clone this repository
4. Copy `.env.example` into `.env` and fill out the empty environment variables
    - You can leave the rest the way they are
    - I recommend changing the `_TAG` variables to `dev` if you plan to change the code
      - If you do so, you will need to build the container using `docker compose build byers`, which can take quite some time and consume a fair amount of resources!
5. Spin up the database using `docker compose up -d db`
6. Migrate the database using `cargo runm`
    - If you changed the Postgres variables (starting with `PG`), you will have to use
      `cargo sqlx migrate run --source judeharley/migrations -D postgres://[PG_USER]:[PG_PASSWORD]@localhost/[PG_DATABASE]`
    - The same goes for other aliases in this project.
7. Run the initial indexing using `cargo index -p "[RADIO_MUSIC]/playlist.m3u" "[RADIO_MUSIC]"`
    - Like mentioned in 6., if you changed the Postgres variables, this will be `cargo run --package=frohike -- indexing -D postgres://[PG_USER]:[PG_PASSWORD]@localhost/[PG_DATABASE] -p "[RADIO_MUSIC]/playlist.m3u" "[RADIO_MUSIC]"`
8. Finally, you can run the project with `docker compose up`

Once you make changes, in order to update and run the local containers, you run the following commands:

1. `docker compose down`
    - Just pressing `Ctrl-C` is **not** enough, as that will only stop the containers
2. `docker compose build byers`
3. `docker compose up`

## Installation

If you want to run your own instance of the radio, you need to follow step 1, 2, 4 (using the `.env.example` from this repository without cloning) and 8 (although you probably want to start the container using `docker compose up -d`).

As for indexing the files, you will want to run `docker compose exec frohike ./frohike/frohike indexing -D postgres://[PG_USER]:[PG_PASSWORD]@db/[PG_DATABASE] -p "/music/playlist.m3u" "/music"`.

Note that the command differs from the one from the development section in that the database URL is different and that the paths to the music are now /music due to mountpoints.

Another alternative would be to run the `/admin reindex` command on Byers.

If you want to stream to an external Icecast instance instead of the provided one, you can remove the ice service from the `docker-compose.yml` file.
