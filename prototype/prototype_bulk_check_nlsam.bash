#!/usr/bin/env bash

if [ -e "$1" ];then
    A_BD="$1";
    BD="$BIGGUS_DISKUS";
    declare -x BIGGUS_DISKUS="$A_BD";
    echo "Using alternate search location $A_BD";
    shift;
fi;

status_scr=/cm/shared/workstation_code_dev/shared/pipeline_utilities/prototype/prototype_nlsam_check_finished.bash

list=$(for r in $@; do 
    r="${r:0:6}NLSAM";
    echo "$r";
done|xargs)
bash $status_scr $list


