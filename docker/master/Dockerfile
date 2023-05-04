FROM frolvlad/alpine-glibc

#RUN apt-get update \
#   && apt-get install -y ca-certificates tzdata \
#  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

ENV RUST_LOG=info

EXPOSE 8080

CMD ["/app/ray-tracer-controller"]
