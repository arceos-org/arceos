RUN_CMD = $(MAKE) -C ../../../ A=$(shell pwd)/../ ARCH=riscv64 run LOG=info

ITER ?= 0
BASE_NAME ?= logs
SUMMARY ?= summary

RAW_LOGS = raw_$(BASE_NAME)_$(ITER).txt
CLEANED_LOGS = $(BASE_NAME)_$(ITER).txt
SUMMARY_CSV = $(SUMMARY)_$(ITER).csv

all: run purify analyze

run:
	$(RUN_CMD) > $(RAW_LOGS)

# purify the ansi chars
purify: run
	sed -r 's/\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]//g' $(RAW_LOGS) | \
	sed -r 's/^\[ [0-9\.]+ [0-9:]+ [^\]]+\] (.*)/\1/' > $(CLEANED_LOGS)

analyze:
	python3 analysis.py $(CLEANED_LOGS) $(SUMMARY_CSV)

clean:
	rm -f $(CLEANED_LOGS) $(RAW_LOGS)

.PHONY: run purify analyze clean