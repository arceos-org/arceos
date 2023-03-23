SQLITE3_CFLAGS := -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION -DSQLITE_DEBUG

app-objs := main.o sqlite3.o

$(APP)/main.o: $(APP)/sqlite3.c

$(APP)/sqlite3.o: $(APP)/sqlite3.c
	$(CC) $(CFLAGS) $(SQLITE3_CFLAGS) -w -c -o $@ $<

# Download sqlite source code
$(APP)/sqlite3.c:
	echo "Download sqlite source code"
	wget https://sqlite.org/2023/sqlite-amalgamation-3410100.zip
	unzip sqlite-amalgamation-3410100.zip
	rm sqlite-amalgamation-3410100/shell.c sqlite-amalgamation-3410100/sqlite3ext.h
	cp sqlite-amalgamation-3410100/* $(APP)
	rm -rf sqlite-amalgamation-3410100 sqlite-amalgamation-3410100.zip
