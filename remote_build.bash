#!/usr/bin/env bash

for hst in $@; do
    ssh $hst 'bash -c "cd $WORKSTATION_CODE/archive/pipe_status;git stash; git pull; git stash pop; declare -x CARGO_NET_GIT_FETCH_WITH_CLI=true; ./build.bash"' &
done
wait;
