################################
#### Download mimirtool
FROM alpine:3.23 as downloader

RUN apk add --no-cache curl

RUN curl -L -o /usr/local/bin/mimirtool https://github.com/grafana/mimir/releases/latest/download/mimirtool-linux-amd64 && \
    chmod +x /usr/local/bin/mimirtool

################################
#### Runtime
FROM alpine:3.23 as runtime

ARG BINARY_PATH=target/x86_64-unknown-linux-musl/release/mimir-cardinality-analyzer

# Create a non-root user
RUN addgroup -S appadmin -g 1000 && adduser -S appadmin -G appadmin -D -u 1000

# Create writable directory and app directory
RUN mkdir -p /data && chown appadmin:appadmin /data
RUN mkdir -p /app && chown appadmin:appadmin /app

WORKDIR /data

# Don't touch these
ENV LC_COLLATE en_US.UTF-8
ENV LC_CTYPE UTF-8
ENV LC_MESSAGES en_US.UTF-8
ENV LC_MONETARY en_US.UTF-8
ENV LC_NUMERIC en_US.UTF-8
ENV LC_TIME en_US.UTF-8
ENV LC_ALL en_US.UTF-8
ENV LANG en_US.UTF-8

# Copy the binary
COPY ${BINARY_PATH} /usr/local/bin/mimir-cardinality-analyzer

RUN chmod +x /usr/local/bin/mimir-cardinality-analyzer
RUN chown appadmin:appadmin /usr/local/bin/mimir-cardinality-analyzer

# Copy mimirtool from downloader stage
COPY --from=downloader /usr/local/bin/mimirtool /usr/local/bin/mimirtool
RUN chown appadmin:appadmin /usr/local/bin/mimirtool

# Run as non-root
USER appadmin
CMD ["/usr/local/bin/mimir-cardinality-analyzer", "--config", "/app/config.yaml"]
