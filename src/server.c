#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <stdint.h>

#define PORT 8081
#define BUFFER_SIZE 2048


//ITS BING ENDIAN, LATER USE HTONS FOR ALL
typedef struct __attribute__((packed)) modbus_tcp{
    uint16_t transaction_id;
    uint16_t protocol_id; //protoc modbus tcp is always 0
    uint16_t lenght; //numb of remainin gytes, 
    uint8_t unit_id; //slave/ server device addr
};



int main(){
    char client_ip[16];
    int serv_d, new_socket;
    struct sockaddr_in address, addr_for_client;
    int opt =1;
    int addrlen = sizeof(addr_for_client);
    char buff[BUFFER_SIZE] = {0};
    

    if ((serv_d = socket(AF_INET, SOCK_STREAM, 0)) < 0)
    {
        perror("socket() error");
        exit(EXIT_FAILURE);
    }


    // CAN REUSE SOCKET AND DONT CLOSE THE PORT
    if (setsockopt(serv_d, SOL_SOCKET, SO_REUSEADDR,  &opt, sizeof(opt)))
    {
        perror("setsockopt() error");
        exit(EXIT_FAILURE);
    }

    // setup address
    address.sin_family = AF_INET; // ipv4
    address.sin_addr.s_addr = INADDR_ANY; // 0.0.0.0 
    address.sin_port = htons(PORT);

    if(bind(serv_d, (struct sockaddr*)&address, sizeof(address)) < 0)
    {
        perror("bind() error");
        exit(EXIT_FAILURE);

    }

    if(listen(serv_d, 3) < 0)
    {
        perror("listen() error");
        exit(EXIT_FAILURE);
    }
    
printf("listen success\n");

addr_for_client.sin_family = AF_INET;

while(1==1){
    //accept connect
if ((new_socket = accept(serv_d, (struct sockaddr*)&addr_for_client, (socklen_t*)&addrlen)) < 0 ){
        perror("accept() error");
        exit(EXIT_FAILURE);
    }

//GET CLIENT IP FROM BIN TO STR
inet_ntop(AF_INET, &addr_for_client.sin_addr, client_ip, sizeof(client_ip));


ssize_t valread;
while((valread = read(new_socket, buff, BUFFER_SIZE)) >0)
{

    struct modbus_tcp* header = (struct modbus_tcp*)buff;
    uint16_t transaction_id = htons(header->transaction_id);
    uint16_t protocol_id = htons(header->protocol_id);
    uint16_t lenght = htons(header->lenght);
    uint8_t unit_id = header->unit_id;// dont need htons for header->unit_id cuz only 1 byte  

    printf("PARSE:\ntrans_id: %d\nprot_id: %d\n lenght: %d\nunit_id: %d\n",
            transaction_id, protocol_id, lenght, unit_id);
    
    int data_len = lenght -1;

    uint8_t data[253] = {0};

    printf("data: ");
    memmove(data, buff+7, data_len);
    for(int i =0; i<data_len; i++){
        printf("%02x", data[i]);
    }
    printf("\n");
    printf("client IP: %s\n", client_ip);
    memset(buff, 0, sizeof(buff));
    uint8_t answer[] = "read succesfull ";
    send(new_socket, answer, sizeof(answer), 0);
}

close(new_socket);
}
close(serv_d);
return 0;   
}




