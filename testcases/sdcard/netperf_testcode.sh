ip="127.0.0.1"
port=12865

run_netperf() {
    echo "====== netperf $1 begin ======"
    ./netperf -H $ip -p $port -t $1 -l 1 -- $2
    if [ $? == 0 ]; then
	    ans="success"
    else
	    ans="fail"
    fi
  echo "====== netperf $1 end: $ans ======"
}

./netserver -D -L $ip -p $port &
server_pid=$!

run_netperf UDP_STREAM  "-s 16k -S 16k -m 1k -M 1k"
run_netperf TCP_STREAM  "-s 16k -S 16k -m 1k -M 1k"
run_netperf UDP_RR      "-s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1"
run_netperf TCP_RR      "-s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1"
run_netperf TCP_CRR     "-s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1"

kill -9 $server_pid
