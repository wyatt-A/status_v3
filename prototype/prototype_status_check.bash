#!/usr/bin/env bash

# /cm/shared/workstation_code_dev/archive/specimen_status/prototype/

# generalize status reporting issue

# stages of work, their % completion, weighting for each stage in regards to others.
# each pipeline defines a "status.conf" 
# status.conf contains a definition of stages
# stages reference some file as a marker for completion
#
# example for co_reg(in less than ideal stage format for simplicity right now)
# pipe=co_reg
# stages=2+
# stage1=mk_nhdr
# stage1_tasks=n-input
# stage1_files=inputs/.*nhdr
#
# stage2=antsRegistration
# stage2_tasks=n-input
# stage1_files=results/.*[Aa]ffine.(mat|txt)
#
# stage3=antsApplyTransform
# stage3_tasks=n-input
# stage3_files=results/Reg_.*nhdr
# 
# stageFinal=write_headfile
# stageFinal_tasks=1
# stageFinal_files=results/pipe_input1.headfile


