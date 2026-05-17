CC      = gcc
CFLAGS  = -Wall -Wextra -O2 -I../../include
LDFLAGS =

ifeq ($(OS),Windows_NT)
  LIB     = ../../bmb_json.dll
  LDFLAGS += -L../.. -l:bmb_json.dll
else
  UNAME := $(shell uname -s)
  ifeq ($(UNAME),Darwin)
    LIB   = ../../libbmb_json.dylib
    LDFLAGS += -L../.. -lbmb_json -Wl,-rpath,../..
  else
    LIB   = ../../libbmb_json.so
    LDFLAGS += -L../.. -lbmb_json -Wl,-rpath,../..
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
