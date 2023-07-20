#!/usr/bin/env bash

# sneaking a rel build for local, schedueld last
#host_list="delos civmcluster1 civmcluster2 vidconfmac piper localhost";
host_list="delos civmcluster2 vidconfmac piper localhost";
if [ ! -z "$@" ];
then
   host_list="$@";
fi;
# ;)
best_dev=$(echo $HOSTNAME |grep -c seba);
if [ "$best_dev" -ge 1 ];
then
   TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl && cargo build --target=x86_64-pc-windows-gnu --release && cargo build --release;
   scp ./target/release/{pipe_status,pipe_status_server} delos:/Volumes/workstation_home/software/bin &
   scp ./target/x86_64-unknown-linux-musl/release/{pipe_status,pipe_status_server} civmcluster1:/cm/shared/workstation_code_dev/bin &
   scp ./target/x86_64-pc-windows-gnu/release/{pipe_status.exe,pipe_status_server.exe} mrs@stejskal:/c/workstation/bin &
else
    # local build of debug only. for giggles. (becuase default build is release.)
    ./build.bash debug &
    ./remote_build.bash $host_list &
    wait
    # if we need to, fetch from our friend's configdir
    # scp -p seba:/Users/Wyatt/IdeaProjects/status_v3/pipe_configs/{bart_recon,acquisition}.toml $WKS_SETTINGS/status_configs
    # cant decide on method.... 
    # rsync -blurtEDv seba:/Users/Wyatt/IdeaProjects/status_v3/pipe_configs/ $WKS_SETTINGS/status_configs/
    # updates repo-configs from local versions
    ./config_update_from_local.bash
fi;
# rsync repo configs to remote locations
./send_config.bash $host_list &
wait
echo "deployment complete"
