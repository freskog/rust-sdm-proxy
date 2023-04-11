FROM rust:1-bookworm

# Install packages
RUN apt-get update && \
    apt-get install -y \
    libglib2.0-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-rtsp \
    libgstrtspserver-1.0-dev \
    gstreamer1.0-tools \
    curl \
    git

VOLUME /root

RUN rustup default stable

CMD ["/bin/bash"]
