# modify compile flags of gcc
CFLAGS +=

# Keep it alive, don't modify it
all:

ifeq (,$(wildcard $(app_path)/sqlite3.c))	# check sqlite3.c exists
  app_src += $(app_path)/sqlite3.c
  app_obj += $(obj_dir)/sqlite3.o
endif

# Download sqlite source code
$(app_path)/sqlite3.%:
	echo "Download sqlite source code"
	wget https://sqlite.org/2023/sqlite-amalgamation-3410100.zip
	unzip sqlite-amalgamation-3410100.zip
	rm sqlite-amalgamation-3410100/shell.c sqlite-amalgamation-3410100/sqlite3ext.h
	cp sqlite-amalgamation-3410100/* $(app_path)
	rm -rf sqlite-amalgamation-3410100 sqlite-amalgamation-3410100.zip

$(obj_dir)/main.o: $(app_path)/main.c $(app_path)/sqlite3.h

# compile sqlite3
$(obj_dir)/sqlite3.o: $(app_path)/sqlite3.c
	$(CC) $(CFLAGS) -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION -DSQLITE_DEBUG -w -c -o $@ $<
