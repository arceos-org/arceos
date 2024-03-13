host="127.0.0.1"
port="5001"
iperf="./iperf3"

run_iperf() {
    name=$1
    args=$2
    echo "====== iperf $name begin ======"

    $iperf -c $host -p $port -t 2 -i 0 $args
    if [ $? == 0 ]; then
	    ans="success"
    else
	    ans="fail"
    fi

    echo "====== iperf $name end: $ans ======"
    echo ""
}


#start server
$iperf -s -p $port -D

#basic test 
run_iperf "BASIC_UDP" "-u -b 1000G" 
run_iperf "BASIC_TCP" ""

#parallel test
run_iperf "PARALLEL_UDP" "-u -P 5 -b 1000G"
run_iperf "PARALLEL_TCP" "-P 5"

#reverse test (server sends, client recieves)
run_iperf "REVERSE_UDP" "-u -R -b 1000G"
run_iperf "REVERSE_TCP" "-R"
