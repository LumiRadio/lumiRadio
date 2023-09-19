FROM savonet/liquidsoap:v2.2.1

USER root

RUN chown -R liquidsoap:liquidsoap /usr/share/liquidsoap

USER liquidsoap

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/bin/liquidsoap"]