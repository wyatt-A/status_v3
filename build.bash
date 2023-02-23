#!/usr/bin/env bash
set -e
# Tried to set up cross-compiling,
# but it's a giant hassle(from linux to mac or win), will probably do remote builds.

build_logs=build_logs
if [ ! -e "$build_logs" ];
then
   mkdir "$build_logs";
fi;
b_start="$build_logs/build_start";
built_log="$build_logs/built.log";
last_built_log="$build_logs/built_last.log";
touch "$b_start";
cargo build  --release &> "$build_logs/release.log" &
cargo build  --debug &> "$build_logs/debug.log" &
wait;

#TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl\
#    && cargo build --target=x86_64-pc-windows-gnu --release\
#    && cargo build --release

find target -newer "$b_start" -and \( -name "pipe_status" -or -name "pipe_status_server" \) >> "$built_log";
bc=$(cat "$built_log"| wc -l );
rm "$b_start";
if [ "$bc" -ge 1 ];
then
    echo "Build complete";
    mv "$built_log" "$last_built_log";
else
    echo "Targets already up to date.";
    cat "$last_built_log";
fi;

#
# update copy in WKS_BIN ... not 100% this is a good idea
#
if [ -e "$WKS_BIN" ];then
   while read file ; do
       fn=$(basename "$file");
       bc="$WKS_BIN/$fn";
       if [[ ! -e "$bc" ]]
		  || [[ "$bc" -ot "$file" ]]; then
	   cp -p "$file" "$bc";
       fi;
   done < "$last_built_log";
fi;
   
