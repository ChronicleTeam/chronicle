
export DOCKER_HOST=unix:///tmp/podman.sock

podman system service --time=0 ${DOCKER_HOST} &

shuttle run --debug