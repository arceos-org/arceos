#!/bin/bash

#export CC=gcc

./dhry2reg 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench DHRY2 test(lps): "$0}'
./whetstone-double 10 | ./busybox grep -o "COUNT|[[:digit:]]\+.[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+.[[:digit:]]\+" | ./busybox awk '{print "Unixbench WHETSTONE test(MFLOPS): "$0}'
./syscall 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SYSCALL test(lps): "$0}'
./context1 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox tail -n1 | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench CONTEXT test(lps): "$0}'
./pipe 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench PIPE test(lps): "$0}'
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


./looper 20 ./multi.sh 1 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SHELL1 test(lpm): "$0}'
./looper 20 ./multi.sh 8 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SHELL8 test(lpm): "$0}'
./looper 20 ./multi.sh 16 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SHELL16 test(lpm): "$0}'
#./looper 30 dc < ../testdir/dc.dat 2>&1

./arithoh 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench ARITHOH test(lps): "$0}'
./short 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SHORT test(lps): "$0}'
./int 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench INT test(lps): "$0}'
./long 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench LONG test(lps): "$0}'
./float 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FLOAT test(lps): "$0}'
./double 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench DOUBLE test(lps): "$0}'
./hanoi 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench HANOI test(lps): "$0}'
./syscall 10 exec | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench EXEC test(lps): "$0}'
