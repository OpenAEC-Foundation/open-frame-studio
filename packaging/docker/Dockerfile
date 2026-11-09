# syntax=docker/dockerfile:1
#
# Docker image of an OpenAEC tool for the SuperCloud (OpenAEC-Blueprint#61): the
# tool's own Linux release (.deb) in a minimal graphical session, streamed to
# the browser on one HTTP port. A tool copies this folder unchanged and only
# passes its package and program as build arguments; see README.md here.
#
#   docker build \
#     --build-arg APP_DEB=openaec-blueprint_2026.44.1_amd64.deb \
#     --build-arg APP_COMMAND=/usr/bin/openaec-blueprint \
#     --build-arg APP_NAME=openaec-blueprint --build-arg APP_TITLE="OpenAEC Blueprint" \
#     --build-arg APP_VERSION=2026.44.1 \
#     --build-arg APP_SOURCE=https://github.com/OpenAEC-Foundation/OpenAEC-Blueprint \
#     -t ghcr.io/openaec-foundation/openaec-blueprint:2026.44.1 .
#
# The build context holds this folder's files, the .deb and an optional setup.d/
# folder with the tool's own start scripts.
FROM debian:bookworm-slim

ARG APP_DEB
ARG APP_COMMAND
ARG APP_NAME
ARG APP_TITLE
ARG APP_VERSION
ARG APP_SOURCE
ARG APP_LICENSE=""
ARG APP_DESCRIPTION="OpenAEC tool, streamed to the browser"
# Libraries the tool needs beyond the session below, separated by spaces.
ARG EXTRA_PACKAGES=""

LABEL org.opencontainers.image.title="${APP_TITLE}" \
      org.opencontainers.image.description="${APP_DESCRIPTION}" \
      org.opencontainers.image.version="${APP_VERSION}" \
      org.opencontainers.image.source="${APP_SOURCE}" \
      org.opencontainers.image.url="${APP_SOURCE}" \
      org.opencontainers.image.vendor="OpenAEC Foundation" \
      org.opencontainers.image.licenses="${APP_LICENSE}"

# The session: a virtual screen, a window manager without title bars that keeps
# every window full screen, a VNC server on localhost only and noVNC through
# websockify. The libraries are those that Rust apps on winit and Tauri apps
# load at run time; the software renderer needs no GPU.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      tini procps python3 ca-certificates \
      xvfb x11vnc matchbox-window-manager novnc websockify \
      libx11-6 libx11-xcb1 libxcb1 libxcursor1 libxi6 libxrandr2 \
      libxkbcommon0 libxkbcommon-x11-0 libfontconfig1 libfreetype6 fonts-dejavu-core \
      ${EXTRA_PACKAGES} \
 && rm -rf /var/lib/apt/lists/*

# The tool itself, from its own release; apt pulls in what the package declares.
COPY ${APP_DEB} /tmp/app.deb
RUN apt-get update \
 && apt-get install -y --no-install-recommends /tmp/app.deb \
 && rm -rf /var/lib/apt/lists/* /tmp/app.deb \
 && test -x "${APP_COMMAND}"

# noVNC, with a start page that connects at once, and /gezond, which the start
# script fills while the app and the stream run.
RUN mkdir -p /opt/openaec/web /opt/openaec/setup.d \
 && cp -r /usr/share/novnc/. /opt/openaec/web/ \
 && ln -s /tmp/openaec/gezond /opt/openaec/web/gezond
COPY index.html /opt/openaec/web/index.html
COPY openaec-start openaec-gezond /opt/openaec/
COPY setup.d/ /opt/openaec/setup.d/

# A regular user; the user's files in /openaec, the app's settings in /config.
RUN useradd --uid 1000 --create-home --home-dir /home/openaec --shell /bin/bash openaec \
 && mkdir -p /openaec /config /tmp/.X11-unix \
 && chmod 1777 /tmp/.X11-unix \
 && chown openaec:openaec /openaec /config \
 && chmod 755 /opt/openaec/openaec-start /opt/openaec/openaec-gezond

ENV OPENAEC_APP="${APP_COMMAND}" \
    OPENAEC_PORT=3000 \
    OPENAEC_SCREEN=1920x1080 \
    HOME=/openaec \
    XDG_CONFIG_HOME=/config \
    XDG_DATA_HOME=/config/data \
    XDG_STATE_HOME=/config/state \
    XDG_CACHE_HOME=/tmp/openaec/cache \
    ICED_BACKEND=tiny-skia \
    LANG=C.UTF-8

USER openaec
WORKDIR /openaec
VOLUME ["/openaec", "/config"]
EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=3 CMD ["/opt/openaec/openaec-gezond"]
ENTRYPOINT ["/usr/bin/tini", "-g", "--", "/opt/openaec/openaec-start"]
