FROM packages.int.cybee.fr/p/tyr/tyr-oci/rust-nuabee:latest AS builder

WORKDIR /app
# Copy the source code
COPY . /app

# Build the project
RUN cargo build --release

FROM packages.int.cybee.fr/p/tyr/tyr-oci/rust-nuabee:latest

RUN apk add --update nodejs npm

# get the binary from the build image
COPY --from=builder /app/target/release/rust-to-ts /usr/local/bin/rust-to-ts