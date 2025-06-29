load('ext://dotenv', 'dotenv')
dotenv()

docker_build(
    "lumiRadio/radio",
    "."
)
docker_build(
    "lumiRadio/liquidsoap",
    "./docker/liquidsoap"
)
docker_build(
    "lumiRadio/calliope",
    "./calliope",
    build_args={
        "NODE_ENV": "development"
    }
)

docker_compose('./docker-compose.dev.yml')

dc_resource(
    "db",
    labels=["infra"]
)

local_resource(
    "migrate",
    cmd="sea migrate up -u postgres://postgres:abcd1234@localhost/postgres",
    labels=["housekeeping"],
    resource_deps=["db"]
)

radio_path = os.getenv("RADIO_MUSIC")
local_resource(
    "index",
    cmd="cargo run --package=frohike -- indexing -D postgres://postgres:abcd1234@localhost/postgres -p " + radio_path + "/playlist.m3u " + radio_path,
    labels=["housekeeping"],
    resource_deps=["db", "migrate"]
)

dc_resource(
    "redis",
    labels=["infra"]
)

dc_resource(
    "langley",
    labels=["app"],
    resource_deps=["db", "redis", "index", "migrate"]
)

dc_resource(
    "icecast",
    labels=["audio-infra"]
)

dc_resource(
    "liquidsoap",
    labels=["audio-infra"],
    resource_deps=["db", "index", "migrate", "langley", "icecast"]
)

dc_resource(
    "byers",
    labels=["app"],
    resource_deps=["db", "redis", "index", "liquidsoap", "migrate"]
)

dc_resource(
    "caliborn",
    labels=["app"],
    resource_deps=["db"]
)

dc_resource(
    "calliope",
    labels=["app"],
    resource_deps=["caliborn"]
)

