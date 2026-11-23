# Phiogy's line alone, for a Linux server: `phiogy --line`, no window.
# The context is what `script/ship.sh` sends: phikit cut down to the
# crates this program builds, with a workspace file that names only
# them. Phiora stands in front of it (puikit: app/docker-compose.yml,
# the service `phiogy`), so nothing here is opened to the outside.
#
#   docker build -f crates/appkit/app/phiogy/Dockerfile -t phiogy-line .
FROM rust:1.95-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake clang pkg-config libfontconfig-dev libfreetype-dev libssl-dev \
    libzstd-dev libxkbcommon-dev libwayland-dev libx11-xcb-dev libxcb1-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
# A server's build is for running, not for timing: no whole-program
# pass, which is most of a release build's minutes.
ENV CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
    CARGO_PROFILE_RELEASE_DEBUG=false
# No cache is mounted: the server's Docker has no BuildKit, so each
# build is a whole one.
RUN cargo build --release -p phiogy --bin phiogy && cp target/release/phiogy /phiogy \
    && ldd /phiogy > /phiogy.needs

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    libfontconfig1 libfreetype6 libssl3 libzstd1 libxkbcommon0 \
    && rm -rf /var/lib/apt/lists/* && useradd --system --no-create-home phiogy
COPY --from=build /phiogy /usr/local/bin/phiogy
COPY --from=build /phiogy.needs /usr/local/share/phiogy.needs
USER phiogy
# Where it listens is `data/line.tsv` (localhost:4310). The settings it
# takes from the system: PHI_CALL_KEY, the word it shares with Phiora;
# PHI_CALL_ICE, a JSON list of ICE servers; PHI_ORIGINS, the pages that
# may call.
CMD ["phiogy", "--line"]
