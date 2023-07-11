# This is the lumiRadio Liquidsoap configuration file
#
# Usually you shouldn't need to edit this file, because most
# configuration is done via environment variables.

# Load settings from environment
host = getenv(default = "ice", "ICECAST_HOST")
port = int_of_string(default = 8000, getenv(default = "8000", "ICECAST_PORT"))
password = getenv(default = "hackme", "ICECAST_PASSWORD")
mount = getenv(default = "lumiradio", "ICECAST_MOUNT")
langley_url = getenv(default = "http://langley:8000/played", "LANGLEY_URL")

# Log to stdout
log.stdout.set(true)
# Allow telnet access for requesting songs
settings.server.telnet.set(true)
settings.server.telnet.port.set(1234)
settings.server.telnet.bind_addr.set("0.0.0.0")
settings.server.timeout.set(-1.0)
server.telnet()

# Host a server for debug purposes
server.harbor()
interactive.harbor()

# Define interactive variables
v = interactive.float("volume", 1.)

# Persist them
interactive.persistent("/etc/liquidsoap/vars.json")

# Load music directory
playlist = mksafe(playlist("/music"))
# Skip blanks
playlist = blank.skip(playlist)

# Play priority queue first, then the regular song request queue and then the playlist
radio = fallback([request.queue(id = "prioq"), request.queue(id = "srq"), playlist])
# Normalize volume
radio = normalize(radio)
# Set volume
radio = amplify(v, radio)

current_track = ref([])
def on_track(t)
    current_track := t
    filename = t["filename"]
    title = t["title"]
    artist = t["artist"]
    album = t["album"]
    headers = [("Content-Type", "application/json")]
    payload = "{\"filename\": \"" ^ filename ^ "\", \"title\": \"" ^ title ^ "\", \"artist\": \"" ^ artist ^ "\", \"album\": \"" ^ album ^ "\" }"

    print(payload)
    http.post(data = payload, headers = headers, langley_url)
    ()
end
source.on_track(radio, on_track)

def on_current_track(cli)
    t = !current_track
    title = t["title"]
    artist = t["artist"]
    album = t["album"]
    "{\"title\": \"" ^ title ^ "\", \"artist\": \"" ^ artist ^ "\", \"album\": \"" ^ album ^ "\"}"
end
server.register(usage="current_track", description="Show the currently playing track", "current_track", on_current_track)

# Output to Icecast
output.icecast(%mp3, host = host, port = port, password = password, mount = mount, icy_metadata = "true", radio)