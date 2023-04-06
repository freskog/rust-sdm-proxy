FROM alpine:edge


# Install packages
RUN apk update --repository=http://dl-cdn.alpinelinux.org/alpine/edge/testing && \
    apk add --no-cache --repository=http://dl-cdn.alpinelinux.org/alpine/edge/testing \
        glib-dev \
        gstreamer-dev \
        gst-plugins-base-dev \
        gst-plugins-good \
        gst-plugins-bad \
        gst-plugins-ugly \
        gst-rtsp-server-dev \
        curl \
        build-base \
        musl-dev \
        rustup \
        git

VOLUME /root

RUN rustup-init -y --default-toolchain stable

CMD ["/bin/bash"]
