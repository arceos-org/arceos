#! /bin/sh
###############################################################################
#  The BYTE UNIX Benchmarks - Release 3
#          Module: tst.sh   SID: 3.4 5/15/91 19:30:24
#          
###############################################################################
# Bug reports, patches, comments, suggestions should be sent to:
#
#	Ben Smith or Rick Grehan at BYTE Magazine
#	ben@bytepb.UUCP    rick_g@bytepb.UUCP
#
###############################################################################
#  Modification Log:
#
###############################################################################
ID="@(#)tst.sh:3.4 -- 5/15/91 19:30:24";
./busybox sort > sort.$$ < $1
./busybox od sort.$$ | ./busybox sort -n -k 1 > od.$$
./busybox grep the sort.$$ | ./busybox tee grep.$$ | ./busybox wc > wc.$$
./busybox rm sort.$$ grep.$$ od.$$ wc.$$
