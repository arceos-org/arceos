repeats=5
warmup=2
for Concurrency in 1 2 5 10 50 100
do
    echo "----- Concurrency: $Concurrency -----"
    sum=0
    for ((i=1; i<=repeats; i++))
    do
        log=`ab -c $Concurrency -n 100000 -q http://10.0.2.15:5555/ | grep "Requests per second"`
        rps=`echo $log | grep -o "[0-9]*\.[0-9]*"`

        if [ $i -gt $warmup ]
        then
            sum=`echo "$sum + $rps"|bc`
            log="$log  |  SUM = $sum"
        fi
        echo "$i: $log"
        sleep 2
    done
    avg=`echo "scale=3;$sum / ($repeats - $warmup)"|bc`
    echo "Concurrency: $Concurrency  |  AVG: $avg"
    sleep 5
done
