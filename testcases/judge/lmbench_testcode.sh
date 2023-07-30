#!/bin/bash
busybox mkdir -p /var/tmp
busybox touch /var/tmp/lmbench
lmbench_all lat_pipe -P 1
