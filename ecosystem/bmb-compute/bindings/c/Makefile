CC      = gcc
CFLAGS  = -Wall -Wextra -O2 -I../../include
LDFLAGS =

ifeq ($(OS),Windows_NT)
  LIB     = ../../bmb_compute.dll
  LDFLAGS += -L../.. -l:bmb_compute.dll
else
  UNAME := $(shell uname -s)
  ifeq ($(UNAME),Darwin)
    LIB   = ../../libbmb_compute.dylib
    LDFLAGS += -L../.. -lbmb_compute -Wl,-rpath,../..
  else
    LIB   = ../../libbmb_compute.so
    LDFLAGS += -L../.. -lbmb_compute -Wl,-rpath,../..
  endif
endif

.PHONY: all test example clean

all: test example

test: test.c $(LIB)
	$(CC) $(CFLAGS) -o test test.c $(LDFLAGS)

example: example.c $(LIB)
	$(CC) $(CFLAGS) -o example example.c $(LDFLAGS)

run-test: test
	./test

run-example: example
	./example

clean:
	rm -f test example test.exe example.exe
