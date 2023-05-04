#!/usr/bin/env bash
#runnonlsam_file=runno_lists/20.5xfad.01_runnos_BXD77
# input is a file with a space separated selection of RUNNONLSAM
runno_file="$1";

#for a in $@; do echo -- $a; done
if [ ! -f "$1" ];then echo "not a file: $1">&2; exit 1;fi;

#exit 0;

ar_dir=/mnt/nclin-comp-pri.dhe.duke.edu/dusom_civm-atlas
proj=22.kempermann.01
# for every DTINLSAM runno
runnos="";
need_mag="";
need_dif="";
need_con="";
for st in $(cat $runno_file);
do 
    proj_dir=$ar_dir/$proj
    #pull the NLSAm off the runno
    r="${st:0:6}";
    runnos="$runnos$r ";
    # find volume recon status in tmp
    grep "$r" /tmp/recent_recons_20.5xfad.01.log || echo "--- $r";
    # check first and last in archive, diffusion run, connectome run
    #ls -d $proj_dir/{${r}_m00,${r}_m66};
    #ls -d $proj_dir/research/diffusion${st}dsi_studio
    #ls -d $proj_dir/research/connectome${st}dsi_studio
    #ls -d $proj_dir/{${r}_m00,${r}_m66}\
    #      $proj_dir/research/diffusion${st}dsi_studio\
    #      $proj_dir/research/connectome${st}dsi_studio
    if [ -d $proj_dir/${r}_m00 ];then
        vol_n=$(grep 'A_dti_vol' $proj_dir/${r}_m00/${r}_m00.headfile|cut -d '=' -f2);
        let vol_Mn=$vol_n-1;
    fi;
    vc=$(ls -d $proj_dir/{${r}_m00,${r}_m$vol_Mn} 2> /dev/null |wc -l )
    diff=$(ls -d $proj_dir/research/diffusion${st}*dsi_studio 2> /dev/null|wc -l )
    conn=$(ls -d $proj_dir/research/connectome${st}*dsi_studio 2> /dev/null|wc -l);
    echo -n "    archive_status:  ";
    if [ $vc -eq 2 ];
    then
        echo -n "mag "
    else
        echo -n "--- "
        need_mag="$need_mag$r ";
    fi;
    if [ $diff -eq 1 ];
    then
        echo -n "dif ";
    else
        echo -n "--- "
        need_dif="$need_dif$r ";
    fi;
    if [ $conn -eq 1 ];
    then
        echo "con";
    else
        echo "---"
        need_con="$need_con$r ";
    fi;
done

echo "Checked Runnumbers:"
echo "  $runnos";
echo "Missing archive:";
echo "  magnitude: $need_mag";
echo "  diffusion: $need_dif";
echo "  connectome: $need_con";

