sqlite3_pkg := sqlite-amalgamation-3410100
sqlite3_dir := $(APP)/$(sqlite3_pkg)
APP_CFLAGS := -I$(sqlite3_dir) -w \
	-DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION -DSQLITE_DEBUG

ifeq ($(ARCH), riscv64)
  LDFLAGS += --no-relax
endif

app-objs := main.o $(sqlite3_pkg)/sqlite3.o

$(APP)/main.o: $(sqlite3_dir)/sqlite3.c

# Download sqlite source code
$(sqlite3_dir)/sqlite3.c:
	echo "Download sqlite source code"
	wget https://sqlite.org/2023/$(sqlite3_pkg).zip -P $(APP)
	unzip $(APP)/$(sqlite3_pkg).zip -d $(APP) && rm -f $(APP)/$(sqlite3_pkg).zip
