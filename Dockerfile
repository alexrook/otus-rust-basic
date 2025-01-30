FROM mcr.microsoft.com/devcontainers/rust:bookworm

ARG USERNAME
ARG USER_UID
ARG USER_GID

# Add packages
RUN apt-get update
RUN apt-get install -y --no-install-recommends git
RUN apt-get install -y --no-install-recommends telnet

# Setup rust machinery
RUN rustup component add rustfmt
RUN rustup component add clippy
RUN cargo install cargo-expand

# Create the user
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd -s /bin/bash --uid $USER_UID --gid $USER_GID -G vscode,rustlang -m $USERNAME \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

RUN chown -R vscode:rustlang '/usr/local/cargo/'

# ********************************************************
# * Anything else you want to do like clean up goes here *
# ********************************************************

# [Optional] Set the default user. Omit if you want to keep the default as root.
USER $USERNAME
