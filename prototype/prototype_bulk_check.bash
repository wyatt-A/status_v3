#!/usr/bin/env bash
# a project code, should be whole thing NOT partial!
proj="$1";
# a file with a whitespace separated selection of RUNNO.
# They can have NLSAM or not.
# This input is optional.
runno_file="$2";


if [ -z "$proj" ];then echo "specify project code first"; fi;
# recon log is optional BUT it set's the final volume looked for!
# this is to avoid complicated checks on local work folders.
recon_log=${HOME}/CS_recon_status/recent_recons_$proj.log

if [ ! -f "$2" ];
then
    #
    if [ "$2" == "" ];then
        if [ ! -e "$recon_log" ];then
            find_recent_recons "$proj" /privateShares/cof/ > $recon_log
        fi;
        #this would only show fully reconstructed
        #status_files=$(grep 100.00 $recon_log|awk '{print $9}'|sort -u);
        status_files=$(awk '{print $9}' $recon_log|sort -u);
        runno_file=${HOME}/CS_recon_status/$proj.list
        for f in ${status_files}; do echo ${f%.*};done > $runno_file
    else
        echo "not a file: $2">&2;
        exit 1;
    fi;
fi;

MIN_DIFF=7;
ar_dir=/mnt/nclin-comp-pri.dhe.duke.edu/dusom_civm-atlas
runnos="";
need_mag="";
need_dif="";
need_con="";
for st in $(cat $runno_file);
do
    proj_dir=$ar_dir/$proj
    #pull the NLSAM off the runno if it's there
    r="${st:0:6}";
    runnos="$runnos$r ";
    # find volume recon status in tmp
    log_line=$(grep "$r" "$recon_log" 2> /dev/null|tail -n1)
    log_search_fail=$?;
    count=1;
    if [ $log_search_fail -eq 0 ];then
        echo "$log_line";
        # get N=number from the log_line
        count=$(echo $log_line|sed -E 's/.*N=([0-9]+).*/\1/')
        let mlast=$count-1;
        read mzero mlast < <(seq -w 0 $mlast $mlast|xargs)
    else
        echo "--- $r";
    fi;
    # check first and last in archive, diffusion run, connectome run
    #ls -d $proj_dir/{${r}_m00,${r}_m66};
    #ls -d $proj_dir/research/diffusion${st}dsi_studio
    #ls -d $proj_dir/research/connectome${st}dsi_studio
    #ls -d $proj_dir/{${r}_m00,${r}_m66}\
    #      $proj_dir/research/diffusion${st}dsi_studio\
    #      $proj_dir/research/connectome${st}dsi_studio
    vc=$(ls -d $proj_dir/{${r}_m${mzero},${r}_m${mlast}} 2> /dev/null |wc -l )
    if [ $count -ge $MIN_DIFF ];then
        diff=$(ls -d $proj_dir/research/diffusion${r}*dsi_studio 2> /dev/null|wc -l )
        conn=$(ls -d $proj_dir/research/connectome${r}*dsi_studio 2> /dev/null|wc -l);
    else
        unset diff;
        unset conn;
    fi;
    echo -n "    archive_status:  ";
    if [ $vc -eq 2 ];
    then
        echo -n "mag "
    else
        echo -n "--- "
        need_mag="$need_mag$r ";
    fi;
    if [ "$diff" != "" ];then
        if [ $diff -eq 1 ];
        then
            echo -n "dif ";
        else
            echo -n "--- "
            need_dif="$need_dif$r ";
        fi;
    fi;
    if [ "$conn" != "" ];then
        if [ $conn -eq 1 ];
        then
            echo "con";
        else
            echo "---"
            need_con="$need_con$r ";
        fi;
    fi;
    if [ $count -lt $MIN_DIFF ];then
        echo "";
    fi;
done

echo "Checked Runnumbers:"
echo "  $runnos";
echo "Missing archive:";
echo "  magnitude: $need_mag";
echo "  diffusion: $need_dif";
echo "  connectome: $need_con";
