.PHONY: all clean

all:
	for i in 1 2; do \
		$(CC) test$$i.c -static -O3 -Wall -o interrupts-test-$$i.o; \
	done

clean:
	rm -f interrupts-test-*
