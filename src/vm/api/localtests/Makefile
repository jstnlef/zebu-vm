# Copyright 2017 The Australian National University
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
