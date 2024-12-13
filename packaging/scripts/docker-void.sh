# For glibc
docker run -it --rm -v $(pwd):/workspace ghcr.io/void-linux/void-linux:latest-full-x86_64 sh

# For musl
docker run -it --rm -v $(pwd):/workspace ghcr.io/void-linux/void-linux:latest-full-x86_64-musl sh
