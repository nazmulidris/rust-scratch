#!/usr/bin/env fish

# stop all running containers (if any).
set -l running_containers (docker ps -aq)
if test -n "$running_containers"
    docker stop $running_containers
    docker system prune -af
end

# clear all existing images.
set -l images (docker image ls -q)
for image in $images
    docker image rm -f $image
end

# build and run the new docker image.
docker build -t r3bl-cmdr-install .
# docker run --interactive --tty r3bl-cmdr-install
docker run r3bl-cmdr-install
