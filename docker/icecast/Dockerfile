FROM pltnk/icecast2

RUN apt update && apt install -y curl && apt clean && rm -rf /var/lib/apt/lists/*

CMD ["/bin/sh", "-c", "/usr/bin/icecast2 -c /etc/icecast2/icecast.xml"]