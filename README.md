#     modbus-proxy

C server

Rust proxy

Rust client


The proxy runs on port `8080`. When a packet is successfully validated, the proxy forwards it to the C server on port `8081`

#  how to run:
run make

in the first terminal window run ./server 

in the second terminal window run ./proxy

in the third terminal window run ./client
