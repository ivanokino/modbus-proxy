

all:

	rustc src/client.rs
	rustc src/proxy.rs
	gcc -o server.o src/server.c



	