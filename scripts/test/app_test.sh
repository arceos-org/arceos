#!/bin/bash

APP=
ROOT=$(realpath $(dirname $0))/../../
TIMEOUT=60s
EXIT_STATUS=0

S_PASS=0
S_FAILED=1
S_TIMEOUT=2
S_BUILD_FAILED=3

RED_C="\x1b[31;1m"
GREEN_C="\x1b[32;1m"
YELLOW_C="\x1b[33;1m"
CYAN_C="\x1b[36;1m"
BLOD_C="\x1b[1m"
END_C="\x1b[0m"

if [ -z "$ARCH" ]; then
    ARCH=x86_64
fi
if [ "$ARCH" != "x86_64" ] && [ "$ARCH" != "riscv64" ] && [ "$ARCH" != "aarch64" ]; then
    echo "Unknown architecture: $ARCH"
    exit $S_FAILED
fi

function compare() {
    local actual=$1
    local expect=$2
    if [ ! -f "$expect" ]; then
        MSG="expected output file \"${BLOD_C}$expect${END_C}\" not found!"
        return $S_FAILED
    fi
    IFS=''
    while read -r line; do
        local matched=$(grep -m1 "$line" < "$actual")
        if [ -z "$matched" ]; then
            MSG="pattern \"${BLOD_C}$line${END_C}\" not matched!"
            unset IFS
            return $S_FAILED
        fi
    done < "$expect"
    unset IFS
    return $S_PASS
}

function run_and_compare() {
    local args=$1
    local expect=$2
    local actual=$3

    echo -ne "    run with \"${BLOD_C}$args${END_C}\": "
    make -C "$ROOT" A="$APP" $args > "$actual" 2>&1
    if [ $? -ne 0 ]; then
        return $S_BUILD_FAILED
    fi

    TIMEFORMAT='%3Rs'
    RUN_TIME=$( { time { timeout --foreground $TIMEOUT make -C "$ROOT" A="$APP" $args justrun > "$actual" 2>&1; }; } 2>&1 )
    local res=$?
    if [ $res == 124 ]; then
        return $S_TIMEOUT
    elif [ $res -ne 0 ]; then
        return $S_FAILED
    fi

    compare "$actual" "$expect"
    if [ $? -ne 0 ]; then
        return $S_FAILED
    else
        return $S_PASS
    fi
}

function test_one() {
    local args=$1
    local expect="$APP/$2"
    local actual="$APP/actual.out"
    args="$args ARCH=$ARCH ACCEL=n"
    rm -f "$actual"

    MSG=
    run_and_compare "$args" "$expect" "$actual"
    local res=$?

    if [ $res -ne $S_PASS ]; then
        EXIT_STATUS=$res
        if [ $res == $S_FAILED ]; then
            echo -e "${RED_C}failed!${END_C} $RUN_TIME"
        elif [ $res == $S_TIMEOUT ]; then
            echo -e "${YELLOW_C}timeout!${END_C} $RUN_TIME"
        elif [ $res == $S_BUILD_FAILED ]; then
            echo -e "${RED_C}build failed!${END_C}"
        fi
        if [ ! -z "$MSG" ]; then
            echo -e "        $MSG"
        fi
        echo -e "${RED_C}actual output${END_C}:"
        cat "$actual"
    else
        echo -e "${GREEN_C}passed!${END_C} $RUN_TIME"
        rm -f "$actual"
    fi
}

if [ -z "$1" ]; then
    test_list=(
        "apps/helloworld"
        "apps/memtest"
        "apps/exception"
        "apps/task/yield"
        "apps/task/parallel"
        "apps/task/sleep"
        "apps/task/priority"
        "apps/task/tls"
        "apps/net/httpclient"
        "apps/c/helloworld"
        "apps/c/memtest"
        "apps/c/sqlite3"
        "apps/c/httpclient"
        "apps/c/pthread/basic"
        "apps/c/pthread/sleep"
        "apps/c/pthread/pipe"
        "apps/c/pthread/parallel"
    )
else
    test_list="$@"
fi

for t in ${test_list[@]}; do
    if [ -z "$1" ]; then
        APP=$(realpath "$ROOT/$t")
    else
        APP=$(realpath "$(pwd)/$t")
    fi
    echo -e "${CYAN_C}Testing${END_C} $t:"
    source "$APP/test_cmd"
done

echo -e "test script exited with: $EXIT_STATUS"
exit $EXIT_STATUS
