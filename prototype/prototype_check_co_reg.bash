#!/usr/bin/env bash
runno="$1";
ls $BIGGUS_DISKUS/co_reg_${runno}_m00-results/*.headfile &>/dev/null|| exit 1;
exit 0;
