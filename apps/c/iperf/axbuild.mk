IPERF_VERSION := 3.1.3
APP_CFLAGS := -Ulinux

iperf_pkg := iperf-$(IPERF_VERSION)
iperf_dir := $(APP)/$(iperf_pkg)
iperf_src := \
	cjson.c \
	iperf_api.c \
	iperf_error.c \
	iperf_client_api.c \
	iperf_locale.c \
	iperf_server_api.c \
	iperf_tcp.c \
	iperf_udp.c \
	iperf_sctp.c \
	iperf_util.c \
	net.c \
	tcp_info.c \
	tcp_window_size.c \
	timer.c \
	units.c \
	main_server.c

app-objs := $(patsubst %.c,$(iperf_pkg)/src/%.o,$(iperf_src))

.PRECIOUS: $(APP)/%.c
$(APP)/%.c:
	@echo "Download iperf source code"
	wget https://downloads.es.net/pub/iperf/$(iperf_pkg).tar.gz -P $(APP)
	tar -zxvf $(APP)/$(iperf_pkg).tar.gz -C $(APP) && rm -f $(APP)/$(iperf_pkg).tar.gz
	cd $(iperf_dir) && git init && git add .
	patch -p1 -N -d $(iperf_dir) --no-backup-if-mismatch -r - < $(APP)/iperf.patch
