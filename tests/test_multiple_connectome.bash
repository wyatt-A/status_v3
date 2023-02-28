#!/usr/bin/bash
# example to validate a list file

# whitespace separated list file

# define a temp file to save our example list to
list_file="$(mktemp -u XXXXXXXXXX.list)";
echo 'S69417
S69415NLAM
S69419' > "$list_file";

st=connectome;
test_dir="$(mktemp -tud ${st}_XXXXXXXXXX_status)";
if [ ! -e "$test_dir" ];
then mkdir "${test_dir}";
fi;
for r in $(cat "$list_file");
do r=${r:0:6}NLSAM;
   pipe_status check $r diffusion_calc_connectome --pipe-configs $WKS_SETTINGS/status_configs |tee "${test_dir}"/"$r.status" &
done
wait;
grep -R -A1  'label": "diffusion_calc_connectome' ${test_dir} | grep Complete
rm "$list_file";
rm -rf "$test_dir";
