#!/usr/bin/env bash
runno="$1";
ls $BIGGUS_DISKUS/diffusion${runno}NLSAMdsi_studio-work/nii4D_${runno}_mask_cropped.nii &>/dev/null|| exit 1;
exit 0;
