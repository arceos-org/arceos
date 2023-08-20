#!/bin/bash
busybox echo iozone automatic measurements
iozone -a -r 1k -s 4m
busybox echo iozone throughput write/read measurements
iozone -t 4 -i 0 -i 1 -r 1k -s 1m
busybox echo iozone throughput random-read measurements
iozone -t 4 -i 0 -i 2 -r 1k -s 1m
busybox echo iozone throughput read-backwards measurements
iozone -t 4 -i 0 -i 3 -r 1k -s 1m
busybox echo iozone throughput stride-read measurements
iozone -t 4 -i 0 -i 5 -r 1k -s 1m
busybox echo iozone throughput fwrite/fread measurements
iozone -t 4 -i 6 -i 7 -r 1k -s 1m
busybox echo iozone throughput pwrite/pread measurements
iozone -t 4 -i 9 -i 10 -r 1k -s 1m
busybox echo iozone throughtput pwritev/preadv measurements
iozone -t 4 -i 11 -i 12 -r 1k -s 1m
