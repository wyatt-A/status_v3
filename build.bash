 #!/usr/bin/env bash
set -e
# Tried to set up cross-compiling,
# but it's a giant hassle(from linux to mac or win), will probably do remote builds.

# release debug or rnd for both
if [ ! -z "$1" ];
then
    type="$1";
else
    type="release";
fi;
if [[ "$type" != "debug" ]] && [[ "$type" != "release" ]] && [[ "$type" != "rnd" ]];
then
    echo "Unknown build type: \"$type\". Will proceede in release build";
    type="release";
fi;
#echo $PATH
#which cargo
cargo=$(which cargo);
if [ -z "$cargo" ];
then
    echo "Cannot find cargo to run the build" >&2;
    exit 1;
fi;
build_logs=build_logs
if [ ! -e "$build_logs" ];
then
   mkdir "$build_logs";
fi;
b_start="$build_logs/build_start";
built_log="$build_logs/built.log";
last_built_log="$build_logs/built_last.log";
touch "$b_start";
if [[ "$type" == "release" ]] || [[ "$type" == "rnd" ]];
then
    #echo "cargo build --release"
    cargo build --release &> "$build_logs/release.log" &
fi
if [[ "$type" == "debug" ]] || [[ "$type" == "rnd" ]];
then
    #echo "cargo build"
    cargo build &> "$build_logs/debug.log" &
fi;
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
elif [ ! -s "$build_logs/release.log" ];
then
    echo "Targets already up to date.";
    rm "$built_log";
    #cat "$last_built_log";
else
    cat "$build_logs/debug.log";
fi;

#
# update copy in WKS_BIN ... not 100% this is a good idea
#
if [ -e "$WKS_BIN" ];
then
    while read file ;
    do
	fn=$(basename "$file");
	bc="$WKS_BIN/$fn";
	if [[ ! -e "$bc" ]] || [[ "$bc" -ot "$file" ]];
	then
	    cp -p "$file" "$bc";
	fi;
    done < "$last_built_log";
fi;
