#!/bin/bash

#export CC=gcc

./spawn 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SPAWN test(lps): "$0}'
UB_BINDIR=./ ./execl 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench EXECL test(lps): "$0}'

#./fstime 
./fstime -w -t 20 -b 256 -m 500 | ./busybox grep -o "WRITE COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_WRITE_SMALL test(KBps): "$0}'
./fstime -r -t 20 -b 256 -m 500 | ./busybox grep -o "READ COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_READ_SMALL test(KBps): "$0}'
./fstime -c -t 20 -b 256 -m 500 | ./busybox grep -o "COPY COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_COPY_SMALL test(KBps): "$0}'

./fstime -w -t 20 -b 1024 -m 2000 | ./busybox grep -o "WRITE COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_WRITE_MIDDLE test(KBps): "$0}'
./fstime -r -t 20 -b 1024 -m 2000 | ./busybox grep -o "READ COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_READ_MIDDLE test(KBps): "$0}'
./fstime -c -t 20 -b 1024 -m 2000 | ./busybox grep -o "COPY COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_COPY_MIDDLE test(KBps): "$0}'

./fstime -w -t 20 -b 4096 -m 8000 | ./busybox grep -o "WRITE COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_WRITE_BIG test(KBps): "$0}'
./fstime -r -t 20 -b 4096 -m 8000 | ./busybox grep -o "READ COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_READ_BIG test(KBps): "$0}'
./fstime -c -t 20 -b 4096 -m 8000 | ./busybox grep -o "COPY COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FS_COPY_BIG test(KBps): "$0}'
