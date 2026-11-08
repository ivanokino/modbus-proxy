all:
	cargo build
	cp target/debug/proxy ./proxy
	cp target/debug/client ./client
	gcc -o server src/env/server.c

clean:
	rm -f proxy client server
	
rel:
	cargo build --release
	cp target/release/proxy ./proxy
	cp target/release/client ./client
	gcc -o server src/env/server.c

