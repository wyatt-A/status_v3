#!/usr/bin/env bash

for hst in delos civmcluster1 vidconfmac piper; do
    ssh $hst 'bash -c "cd $WORKSTATION_CODE/archive/pipe_status;./build.sh"' &
done
wait;
