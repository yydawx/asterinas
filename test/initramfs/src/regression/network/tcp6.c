// SPDX-License-Identifier: MPL-2.0

#define _GNU_SOURCE

#include <unistd.h>
#include <sys/socket.h>
#include <sys/poll.h>
#include <sys/wait.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>

#include "../common/test.h"

static struct sockaddr_in6 sk_addr;

#define PORT htons(8080)

FN_SETUP(general)
{
	sk_addr.sin6_family = AF_INET6;
	sk_addr.sin6_port = PORT;
	sk_addr.sin6_addr = in6addr_loopback;
}
END_SETUP()

FN_TEST(tcp6_connection)
{
	int server_fd, client_fd, accepted_fd;
	pid_t pid;
	char buffer[1024] = { 0 };
	char *hello = "Hello from client";
	char *response = "Hello from server";
	socklen_t addrlen = sizeof(sk_addr);

	// Create server socket
	server_fd = TEST_SUCC(socket(AF_INET6, SOCK_STREAM, 0));

	// Set socket options
	int opt = 1;
	TEST_SUCC(setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt,
			     sizeof(opt)));

	// Bind the socket
	TEST_SUCC(
		bind(server_fd, (struct sockaddr *)&sk_addr, sizeof(sk_addr)));

	// Listen for connections
	TEST_SUCC(listen(server_fd, 3));

	// Fork a child process for the client
	pid = TEST_SUCC(fork());

	if (pid == 0) {
		// Child process (client)
		close(server_fd);

		// Create client socket
		client_fd = CHECK(socket(AF_INET6, SOCK_STREAM, 0));

		// Connect to server
		CHECK(connect(client_fd, (struct sockaddr *)&sk_addr,
			      sizeof(sk_addr)));

		// Send message to server
		CHECK_WITH(send(client_fd, hello, strlen(hello), 0),
			   _ret == (ssize_t)strlen(hello));

		// Receive response from server
		CHECK_WITH(read(client_fd, buffer, 1024),
			   _ret == (ssize_t)strlen(response));
		CHECK_WITH(strcmp(buffer, response), _ret == 0);

		// Close client socket
		CHECK(close(client_fd));
		_exit(0);
	} else {
		// Parent process (server)
		// Accept connection
		accepted_fd = TEST_SUCC(accept(
			server_fd, (struct sockaddr *)&sk_addr, &addrlen));

		// Read message from client
		TEST_RES(read(accepted_fd, buffer, 1024),
			 _ret == strlen(hello));
		TEST_RES(strcmp(buffer, hello), _ret == 0);

		// Send response to client
		TEST_RES(send(accepted_fd, response, strlen(response), 0),
			 _ret == strlen(response));

		// Close sockets
		TEST_SUCC(close(accepted_fd));
		TEST_SUCC(close(server_fd));

		// Wait for child process to finish
		int status;
		TEST_SUCC(waitpid(pid, &status, 0));
		TEST_RES(WIFEXITED(status), _ret != 0);
		TEST_RES(WEXITSTATUS(status), _ret == 0);
	}
}
END_TEST()
