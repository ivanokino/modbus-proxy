all:
	cargo build
	cp target/debug/proxy ./proxy
	cp target/debug/client ./client
	gcc -o server src/server.c

clean:
	cargo clean
	rm -f proxy client server



	