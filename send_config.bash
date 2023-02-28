#!/usr/bin/env bash

for hst in $@; do
    scp -rp "$PWD/pipe_configs/"*.toml "$hst:$(ssh $hst echo \$WKS_SETTINGS)/status_configs/" &
done
wait;
