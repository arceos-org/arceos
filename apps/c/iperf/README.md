### How to test network performance
1.
```
# in app "iperf" directory
make -C /path/to/arceos A=$(pwd) ARCH=<arch> FS=y NET=y run
# or in arceos directory
make  A=path/to/iperf ARCH=<arch> FS=y NET=y run
```
2. 
```
# In another shell, run iperf UDP client
iperf3 -uc 0.0.0.0 -p 5555 -l 1300 -b <Approximate Bandwidth (50Mb as an example)>
# Or iperf TCP client
iperf3 -c 0.0.0.0 -p 5555 
# Or use UDP reverse mode (server sends, client receives)
iperf3 -uc 0.0.0.0 -p 5555 -l 1300 -b <Approximate Bandwidth (200Mb as an example)> -R
# or TCP reverse mode
iperf3 -c 0.0.0.0 -p 5555 -R
```
