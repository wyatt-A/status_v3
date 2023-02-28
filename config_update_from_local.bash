#!/usr/bin/env bash

# update configs in repo from localy updated ones.
rsync -blurtEDv $WKS_SETTINGS/status_configs/ pipe_configs/
