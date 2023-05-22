# This is the lumiRadio Liquidsoap configuration file
#
# Usually you shouldn't need to edit this file, because most
# configuration is done via environment variables.

# Load settings from environment
host = getenv(default = "ice", "ICECAST_HOST")
port = int_of_string(default = 8000, getenv(default = "8000", "ICECAST_PORT"))
password = getenv(default = "hackme", "ICECAST_PASSWORD")
mount = getenv(default = "lumiradio", "ICECAST_MOUNT")

# Log to stdout
log.stdout.set(true)
# Allow telnet access for requesting songs
settings.server.telnet.set(true)

# Load music directory
playlist = mksafe(playlist("/music"))
# Skip blanks
playlist = blank.skip(playlist)

# Play priority queue first, then the regular song request queue and then the playlist
radio = fallback([request.queue(id = "prioq"), request.queue(id = "srq"), playlist])
# Normalize volume
radio = normalize(radio)

# Output to Icecast
output.icecast(%mp3, host = host, port = port, password = password, mount = mount, icy_metadata = "true", radio)