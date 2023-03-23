AX_ROOT := /home/oslab/OS/arceos

all:
	$(MAKE) -C $(AX_ROOT) A=$(PWD)

run:
	$(MAKE) -C $(AX_ROOT) A=$(PWD) run

clean:
	rm -rf *.o
	$(MAKE) -C $(AX_ROOT) A=$(PWD) clean

.PHONY: all run clean
