#!/bin/bash

#export CC=gcc
./arithoh 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench ARITHOH test(lps): "$0}'
./short 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SHORT test(lps): "$0}'
./int 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench INT test(lps): "$0}'
./long 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench LONG test(lps): "$0}'
./float 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench FLOAT test(lps): "$0}'
./double 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench DOUBLE test(lps): "$0}'
./hanoi 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench HANOI test(lps): "$0}'
./syscall 10 exec | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench EXEC test(lps): "$0}'

./dhry2reg 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench DHRY2 test(lps): "$0}'
./whetstone-double 10 | ./busybox grep -o "COUNT|[[:digit:]]\+.[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+.[[:digit:]]\+" | ./busybox awk '{print "Unixbench WHETSTONE test(MFLOPS): "$0}'
./syscall 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SYSCALL test(lps): "$0}'
./context1 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox tail -n1 | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench CONTEXT test(lps): "$0}'
./pipe 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench PIPE test(lps): "$0}'
./spawn 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench SPAWN test(lps): "$0}'
UB_BINDIR=./ ./execl 10 | ./busybox grep -o "COUNT|[[:digit:]]\+|" | ./busybox grep -o "[[:digit:]]\+" | ./busybox awk '{print "Unixbench EXECL test(lps): "$0}'

