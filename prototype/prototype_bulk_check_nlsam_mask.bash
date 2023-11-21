#!/usr/bin/env bash

runnos=$(echo 'S69421
S69423
S69431
S69433
S69435
S69439
S69441
S69443
S69421
S69423
S69431
S69433
S69435
S69439
S69441
S69443'|sort -u |xargs);

BD=/privateShares/cof;
for runno in $runnos;do
    echo -- $runno --;
    ls $BD/diffusion${runno}NLSAMdsi_studio-work/nii4D_${runno}_mask_cropped.nii 2>/dev/null|| echo "no cropped mask";
done
