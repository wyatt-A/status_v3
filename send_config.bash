#!/usr/bin/env bash

for hst in delos civmcluster1 vidconfmac piper; do
    scp -rp "$PWD/pipe_configs/"* "$hst:$(ssh $hst echo \$WKS_SETTINGS)/status_configs/" &
done
wait;
