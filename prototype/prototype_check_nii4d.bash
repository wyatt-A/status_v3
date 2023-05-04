#!/usr/bin/env bash
runno="$1";
ls $BIGGUS_DISKUS/diffusion${runno}*-results/nii4D_${runno}.nii &>/dev/null|| exit 1;
exit 0;
