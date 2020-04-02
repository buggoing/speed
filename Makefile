.PHONY: all clean build

define echo_green
    printf "\e[38;5;40m"
    echo "${1}"
    printf "\e[0m \n"
endef

current_dir=$(shell pwd)
all: build

clean:
	cargo clean

build:
	cargo build --release
	@${call echo_green,"build finished! The targets is in the ${current_dir}/target/"}

linux: 
	cargo build --release --target x86_64-unknown-linux-musl
	

