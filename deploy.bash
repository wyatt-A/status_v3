#!/usr/bin/env bash

host_list="delos civmcluster1 vidconfmac piper";
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
    ./build.bash $host_list &
    ./remote_build.bash $host_list &
fi;

./send_config.bash $host_list &
wait
echo "deployment complete"
