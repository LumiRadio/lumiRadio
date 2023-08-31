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
# settings.server.telnet.set(true)
# settings.server.telnet.port.set(1234)
# settings.server.telnet.bind_addr.set("0.0.0.0")
settings.server.timeout.set(-1.0)
settings.server.socket.set(true)
settings.server.socket.path.set("/etc/liquidsoap/lumiradio.sock")
settings.server.socket.permissions.set(0o666)

# Define interactive variables
v = interactive.float("volume", 1.)

# Persist them
interactive.persistent("/etc/liquidsoap/vars.json")

# server.telnet()
server.harbor()
interactive.harbor()

# Load music directory
playlist = playlist("/music")
# Skip blanks
playlist = mksafe(blank.eat(playlist))

srq = request.queue(id = "srq")
prioq = request.queue(id = "prioq")

# Play priority queue first, then the regular song request queue and then the playlist
radio = fallback([prioq, srq, playlist])
# Normalize volume
radio = normalize(radio)
# Set volume
radio = amplify(v, radio)

current_track = ref([])
def on_track(t)
    current_track := t
    headers = [("Content-Type", "application/json")]
    payload = json()
    payload.add("artist", t["artist"])
    payload.add("title", t["title"])
    payload.add("album", t["album"])
    payload.add("filename", t["filename"])
    payload_s = json.stringify(payload)

    print(payload_s)
    http.post(data = payload_s, headers = headers, langley_url)
    ()
end
source.on_track(radio, on_track)

def on_current_track(cli)
    t = !current_track
    json.stringify(t)
end
server.register(usage="current_track", description="Show the currently playing track", "current_track", on_current_track)

def on_song_request_queue(cli)
    q = list.append(prioq.queue(), srq.queue())
    json.stringify(q)
end
server.register(usage="song_request_queue", description="Show the song request queue", "song_request_queue", on_song_request_queue)

# Output to Icecast
output.icecast(%mp3, host = host, port = port, password = password, mount = mount, icy_metadata = "true", radio)
