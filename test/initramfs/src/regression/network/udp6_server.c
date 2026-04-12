// SPDX-License-Identifier: MPL-2.0

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>

#define SERVER_PORT 1234
#define BUFFER_SIZE 1024

int main()
{
	int sock_fd;
	char buffer[BUFFER_SIZE];
	struct sockaddr_in6 serv_addr;

	// Create UDP socket
	if ((sock_fd = socket(AF_INET6, SOCK_DGRAM, 0)) < 0) {
		perror("socket creation failed");
		exit(EXIT_FAILURE);
	}

	// Set server address
	memset(&serv_addr, 0, sizeof(serv_addr));
	serv_addr.sin6_family = AF_INET6;
	serv_addr.sin6_port = htons(SERVER_PORT);
	serv_addr.sin6_addr = in6addr_any;

	// Bind to server address
	if (bind(sock_fd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) <
	    0) {
		perror("bind failed");
		exit(EXIT_FAILURE);
	}

	// Receive message from client
	struct sockaddr_in6 sender_addr;
	socklen_t sender_len = sizeof(sender_addr);
	int recv_len;
	if ((recv_len = recvfrom(sock_fd, buffer, BUFFER_SIZE, 0,
				 (struct sockaddr *)&sender_addr,
				 &sender_len)) < 0) {
		perror("recvfrom failed");
		exit(EXIT_FAILURE);
	}
	buffer[recv_len] = '\0';

	char addr_str[INET6_ADDRSTRLEN];
	inet_ntop(AF_INET6, &sender_addr.sin6_addr, addr_str, sizeof(addr_str));
	printf("Received %s from [%s]:%d\n", buffer, addr_str,
	       ntohs(sender_addr.sin6_port));

	// Send message to client
	const char *message = "Hello world from udp6 server!";
	if (sendto(sock_fd, message, strlen(message), 0,
		   (const struct sockaddr *)&sender_addr,
		   sizeof(sender_addr)) < 0) {
		perror("sendto failed");
		exit(EXIT_FAILURE);
	}

	// Close socket
	close(sock_fd);

	return 0;
}