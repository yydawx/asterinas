// SPDX-License-Identifier: MPL-2.0

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>

#define PORT 8080

int main()
{
	int sock = 0;
	struct sockaddr_in6 serv_addr;
	char *hello = "Hello from client";
	char buffer[1024] = { 0 };
	char addr_str[INET6_ADDRSTRLEN];

	printf("[IPv6 Test] Starting TCP6 client test\n");

	// Create socket
	printf("[IPv6 Test] Step 1: Creating IPv6 socket...\n");
	if ((sock = socket(AF_INET6, SOCK_STREAM, 0)) < 0) {
		perror("Socket creation error");
		return -1;
	}
	printf("[IPv6 Test] Socket created successfully, fd: %d\n", sock);

	// Configure server address
	printf("[IPv6 Test] Step 2: Configuring server address...\n");
	serv_addr.sin6_family = AF_INET6;
	serv_addr.sin6_port = htons(PORT);
	serv_addr.sin6_addr = in6addr_loopback;
	
	// Convert IPv6 address to string for display
	if (inet_ntop(AF_INET6, &(serv_addr.sin6_addr), addr_str, INET6_ADDRSTRLEN) != NULL) {
		printf("[IPv6 Test] Connecting to IPv6 address: %s, port: %d\n", addr_str, PORT);
	} else {
		perror("inet_ntop failed");
	}

	// Connect to the server
	printf("[IPv6 Test] Step 3: Connecting to server...\n");
	if (connect(sock, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) {
		perror("Connection failed");
		close(sock);
		return -1;
	}
	printf("[IPv6 Test] Connection established successfully\n");

	// Send message to the server and receive the reply
	printf("[IPv6 Test] Step 4: Sending message to server...\n");
	size_t sent_bytes = send(sock, hello, strlen(hello), 0);
	if (sent_bytes < 0) {
		perror("send failed");
		close(sock);
		return -1;
	}
	printf("[IPv6 Test] Sent %zu bytes: %s\n", sent_bytes, hello);
	
	printf("[IPv6 Test] Step 5: Receiving reply from server...\n");
	ssize_t read_bytes = read(sock, buffer, 1024);
	if (read_bytes < 0) {
		perror("read failed");
		close(sock);
		return -1;
	}
	printf("[IPv6 Test] Read %zd bytes from server\n", read_bytes);
	printf("[IPv6 Test] Server: %s\n", buffer);

	// Cleanup
	printf("[IPv6 Test] Step 6: Closing socket...\n");
	close(sock);
	printf("[IPv6 Test] TCP6 client test completed successfully\n");
	return 0;
}