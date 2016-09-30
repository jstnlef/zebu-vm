RUSTC=rustc
MUAPI_H_DIR=..
INCLUDE_DIR=$(MUAPI_H_DIR)

UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
    LDFLAGS = -lm
else
    LDFLAGS = -ldl -pthread -lm -lrt -lutil
endif

.PHONY: all

all: cclient

rustpart.a: *.rs
	$(RUSTC) --crate-type=staticlib -o rustpart.a mock.rs

cpart.o: cpart.c cpart.h
	$(CC) -c -I $(INCLUDE_DIR) -o $@ $<

cclient: cclient.c cpart.h rustpart.a cpart.o ../muapi.h
	$(CC) -I $(INCLUDE_DIR) $(LDFLAGS) -o $@ cclient.c cpart.o rustpart.a

.PHONY: clean

clean:
	rm *.a *.o cclient
