// SPDX-License-Identifier: MPL-2.0

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>

#define PORT 8080

int main()
{
	int server_fd, new_socket;
	struct sockaddr_in6 address;
	int opt = 1;
	socklen_t addrlen = sizeof(address);
	char buffer[1024] = { 0 };
	char *hello = "Hello from server";
	char addr_str[INET6_ADDRSTRLEN];

	printf("[IPv6 Test] Starting TCP6 server test\n");

	// Create socket
	printf("[IPv6 Test] Step 1: Creating IPv6 socket...\n");
	if ((server_fd = socket(AF_INET6, SOCK_STREAM, 0)) == 0) {
		perror("socket failed");
		exit(EXIT_FAILURE);
	}
	printf("[IPv6 Test] Socket created successfully, fd: %d\n", server_fd);

	// Set socket options
	printf("[IPv6 Test] Step 2: Setting socket options...\n");
	if (setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR | SO_REUSEPORT, &opt,
		       sizeof(opt))) {
		perror("setsockopt failed");
		exit(EXIT_FAILURE);
	}
	printf("[IPv6 Test] Socket options set successfully\n");

	// Configure address
	printf("[IPv6 Test] Step 3: Configuring IPv6 address...\n");
	address.sin6_family = AF_INET6;
	address.sin6_port = htons(PORT);
	address.sin6_addr = in6addr_loopback;
	
	// Convert IPv6 address to string for display
	if (inet_ntop(AF_INET6, &(address.sin6_addr), addr_str, INET6_ADDRSTRLEN) != NULL) {
		printf("[IPv6 Test] Using IPv6 address: %s, port: %d\n", addr_str, PORT);
	} else {
		perror("inet_ntop failed");
	}

	// Bind the socket to specified IP and port
	printf("[IPv6 Test] Step 4: Binding socket...\n");
	if (bind(server_fd, (struct sockaddr *)&address, sizeof(address)) < 0) {
		perror("bind failed");
		exit(EXIT_FAILURE);
	}
	printf("[IPv6 Test] Socket bound successfully\n");

	// Listen for connections
	printf("[IPv6 Test] Step 5: Listening for connections...\n");
	if (listen(server_fd, 3) < 0) {
		perror("listen failed");
		exit(EXIT_FAILURE);
	}
	printf("[IPv6 Test] Listening on port %d\n", PORT);

	// Accept the connection
	printf("[IPv6 Test] Step 6: Waiting for client connection...\n");
	if ((new_socket = accept(server_fd, (struct sockaddr *)&address,
			 &addrlen)) < 0) {
		perror("accept failed");
		exit(EXIT_FAILURE);
	}
	
	// Display client address
	if (inet_ntop(AF_INET6, &(address.sin6_addr), addr_str, INET6_ADDRSTRLEN) != NULL) {
		printf("[IPv6 Test] Client connected from: %s\n", addr_str);
	} else {
		perror("inet_ntop failed for client address");
	}
	printf("[IPv6 Test] Connection accepted, new socket fd: %d\n", new_socket);

	// Read the message from the client and reply
	printf("[IPv6 Test] Step 7: Reading client message...\n");
	ssize_t read_bytes = read(new_socket, buffer, 1024);
	if (read_bytes < 0) {
		perror("read failed");
		close(new_socket);
		close(server_fd);
		exit(EXIT_FAILURE);
	}
	printf("[IPv6 Test] Read %zd bytes from client\n", read_bytes);
	printf("[IPv6 Test] Client: %s\n", buffer);
	
	printf("[IPv6 Test] Step 8: Sending reply...\n");
	send(new_socket, hello, strlen(hello), 0);
	printf("[IPv6 Test] Hello message sent\n");

	// Cleanup
	printf("[IPv6 Test] Step 9: Closing sockets...\n");
	close(new_socket);
	close(server_fd);
	printf("[IPv6 Test] TCP6 server test completed successfully\n");
	return 0;
}