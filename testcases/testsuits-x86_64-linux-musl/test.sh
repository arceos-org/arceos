#!/bin/busybox sh

./lua $1
if [ $? == 0 ]; then
	echo "testcase lua $1 success"
else
	echo "testcase lua $1 fail"
fi
