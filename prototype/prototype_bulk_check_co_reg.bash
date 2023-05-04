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
    ls $BD/co_reg_${runno}_m00-results/*.headfile 2>/dev/null|| echo "no headfile";
done

