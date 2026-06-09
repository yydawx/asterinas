// SPDX-License-Identifier: MPL-2.0

#include <errno.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>
#include "../common/test.h"

FN_TEST(ipv6_v6only_default)
{
	int sk = CHECK(socket(AF_INET6, SOCK_DGRAM, 0));

	int v6only;
	socklen_t v6only_len = sizeof(v6only);
	TEST_RES(getsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			    &v6only_len),
		 v6only == 1);

	close(sk);
}
END_TEST()

FN_TEST(ipv6_v6only_toggle)
{
	int sk = CHECK(socket(AF_INET6, SOCK_DGRAM, 0));

	int v6only = 0;
	TEST_SUCC(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			     sizeof(v6only)));

	v6only = 1;
	socklen_t v6only_len = sizeof(v6only);
	TEST_RES(getsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			    &v6only_len),
		 v6only == 0);

	v6only = 1;
	TEST_SUCC(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			     sizeof(v6only)));

	v6only = 0;
	v6only_len = sizeof(v6only);
	TEST_RES(getsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			    &v6only_len),
		 v6only == 1);

	close(sk);
}
END_TEST()

FN_TEST(ipv6_v6only_reject_on_af_inet)
{
	int sk = CHECK(socket(AF_INET, SOCK_DGRAM, 0));

	int v6only;
	socklen_t v6only_len = sizeof(v6only);
	TEST_ERRNO(getsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			      &v6only_len),
		   ENOPROTOOPT);

	v6only = 0;
	TEST_ERRNO(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			      sizeof(v6only)),
		   ENOPROTOOPT);

	close(sk);
}
END_TEST()

FN_TEST(ipv6_v6only_reject_mapped)
{
	int sk = CHECK(socket(AF_INET6, SOCK_DGRAM, 0));

	struct sockaddr_in6 mapped_addr = { 0 };
	mapped_addr.sin6_family = AF_INET6;
	mapped_addr.sin6_port = htons(8080);
	// ::ffff:127.0.0.1 — IPv4-mapped IPv6, should be rejected when v6only=1
	mapped_addr.sin6_addr.s6_addr[10] = 0xff;
	mapped_addr.sin6_addr.s6_addr[11] = 0xff;
	mapped_addr.sin6_addr.s6_addr[12] = 127;
	mapped_addr.sin6_addr.s6_addr[13] = 0;
	mapped_addr.sin6_addr.s6_addr[14] = 0;
	mapped_addr.sin6_addr.s6_addr[15] = 1;

	TEST_ERRNO(bind(sk, (struct sockaddr *)&mapped_addr,
			sizeof(mapped_addr)),
		   EAFNOSUPPORT);

	close(sk);
}
END_TEST()

FN_TEST(ipv6_dualstack_broadcast_sendto)
{
	int sk = CHECK(socket(AF_INET6, SOCK_DGRAM, 0));

	int v6only = 0;
	TEST_SUCC(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			     sizeof(v6only)));

	struct sockaddr_in broadcast_addr = { 0 };
	broadcast_addr.sin_family = AF_INET;
	broadcast_addr.sin_port = htons(12345);
	broadcast_addr.sin_addr.s_addr = htonl(INADDR_BROADCAST);

	TEST_ERRNO(sendto(sk, "test", 4, 0, (struct sockaddr *)&broadcast_addr,
			  sizeof(broadcast_addr)),
		   EACCES);

	close(sk);
}
END_TEST()

FN_TEST(ipv6_dualstack_broadcast_connect)
{
	int sk = CHECK(socket(AF_INET6, SOCK_DGRAM, 0));

	int v6only = 0;
	TEST_SUCC(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			     sizeof(v6only)));

	struct sockaddr_in broadcast_addr = { 0 };
	broadcast_addr.sin_family = AF_INET;
	broadcast_addr.sin_port = htons(12345);
	broadcast_addr.sin_addr.s_addr = htonl(INADDR_BROADCAST);

	TEST_ERRNO(connect(sk, (struct sockaddr *)&broadcast_addr,
			   sizeof(broadcast_addr)),
		   EACCES);

	close(sk);
}
END_TEST()

FN_TEST(ipv6_dualstack_connect)
{
	int sk = CHECK(socket(AF_INET6, SOCK_DGRAM, 0));

	int v6only = 0;
	TEST_SUCC(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			     sizeof(v6only)));

	struct sockaddr_in v4_addr = { 0 };
	v4_addr.sin_family = AF_INET;
	v4_addr.sin_port = htons(8080);
	v4_addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

	TEST_SUCC(connect(sk, (struct sockaddr *)&v4_addr, sizeof(v4_addr)));

	struct sockaddr_in6 peer = { 0 };
	socklen_t peer_len = sizeof(peer);
	TEST_SUCC(getpeername(sk, (struct sockaddr *)&peer, &peer_len));

	// Must present as IPv4-mapped IPv6 (::ffff:127.0.0.1)
	TEST_RES(peer.sin6_family, peer.sin6_family == AF_INET6);
	TEST_RES(ntohs(peer.sin6_port), ntohs(peer.sin6_port) == 8080);
	TEST_RES(peer.sin6_addr.s6_addr[10],
		 peer.sin6_addr.s6_addr[10] == 0xff);
	TEST_RES(peer.sin6_addr.s6_addr[11],
		 peer.sin6_addr.s6_addr[11] == 0xff);
	TEST_RES(peer.sin6_addr.s6_addr[12], peer.sin6_addr.s6_addr[12] == 127);
	TEST_RES(peer.sin6_addr.s6_addr[13], peer.sin6_addr.s6_addr[13] == 0);
	TEST_RES(peer.sin6_addr.s6_addr[14], peer.sin6_addr.s6_addr[14] == 0);
	TEST_RES(peer.sin6_addr.s6_addr[15], peer.sin6_addr.s6_addr[15] == 1);

	// getsockname must also return IPv4-mapped IPv6 per RFC 4038
	struct sockaddr_in6 local = { 0 };
	socklen_t local_len = sizeof(local);
	TEST_SUCC(getsockname(sk, (struct sockaddr *)&local, &local_len));
	TEST_RES(local.sin6_family, local.sin6_family == AF_INET6);

	close(sk);
}
END_TEST()

FN_TEST(ipv6_dualstack_sendto)
{
	int sk = CHECK(socket(AF_INET6, SOCK_DGRAM, 0));

	int v6only = 0;
	TEST_SUCC(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			     sizeof(v6only)));

	struct sockaddr_in v4_addr = { 0 };
	v4_addr.sin_family = AF_INET;
	v4_addr.sin_port = htons(8080);
	v4_addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

	// Must not fail with EAFNOSUPPORT on dual-stack socket
	TEST_SUCC(sendto(sk, "test", 4, 0, (struct sockaddr *)&v4_addr,
			 sizeof(v4_addr)));

	close(sk);
}
END_TEST()

// ---- Stream socket tests ----

FN_TEST(stream_ipv6_v6only_default)
{
	int sk = CHECK(socket(AF_INET6, SOCK_STREAM, 0));

	int v6only;
	socklen_t v6only_len = sizeof(v6only);
	TEST_RES(getsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			    &v6only_len),
		 v6only == 1);

	close(sk);
}
END_TEST()

FN_TEST(stream_ipv6_v6only_reject_on_af_inet)
{
	int sk = CHECK(socket(AF_INET, SOCK_STREAM, 0));

	int v6only;
	socklen_t v6only_len = sizeof(v6only);
	TEST_ERRNO(getsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			      &v6only_len),
		   ENOPROTOOPT);

	v6only = 0;
	TEST_ERRNO(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			      sizeof(v6only)),
		   ENOPROTOOPT);

	close(sk);
}
END_TEST()

FN_TEST(stream_ipv6_v6only_reject_mapped_bind)
{
	int sk = CHECK(socket(AF_INET6, SOCK_STREAM, 0));

	struct sockaddr_in6 mapped_addr = { 0 };
	mapped_addr.sin6_family = AF_INET6;
	mapped_addr.sin6_port = htons(8080);
	mapped_addr.sin6_addr.s6_addr[10] = 0xff;
	mapped_addr.sin6_addr.s6_addr[11] = 0xff;
	mapped_addr.sin6_addr.s6_addr[12] = 127;
	mapped_addr.sin6_addr.s6_addr[13] = 0;
	mapped_addr.sin6_addr.s6_addr[14] = 0;
	mapped_addr.sin6_addr.s6_addr[15] = 1;

	TEST_ERRNO(bind(sk, (struct sockaddr *)&mapped_addr,
			sizeof(mapped_addr)),
		   EAFNOSUPPORT);

	close(sk);
}
END_TEST()

FN_TEST(stream_ipv6_dualstack_bind)
{
	int sk = CHECK(socket(AF_INET6, SOCK_STREAM, 0));

	int v6only = 0;
	TEST_SUCC(setsockopt(sk, IPPROTO_IPV6, IPV6_V6ONLY, &v6only,
			     sizeof(v6only)));

	struct sockaddr_in6 bind_addr = { 0 };
	bind_addr.sin6_family = AF_INET6;
	bind_addr.sin6_port = htons(0);
	bind_addr.sin6_addr = in6addr_loopback;
	TEST_SUCC(bind(sk, (struct sockaddr *)&bind_addr, sizeof(bind_addr)));
	TEST_SUCC(listen(sk, 1));

	// Verify getsockname returns correct family and a valid port
	struct sockaddr_in6 actual_addr = { 0 };
	socklen_t addr_len = sizeof(actual_addr);
	TEST_SUCC(getsockname(sk, (struct sockaddr *)&actual_addr, &addr_len));
	TEST_RES(actual_addr.sin6_family, actual_addr.sin6_family == AF_INET6);
	TEST_RES(ntohs(actual_addr.sin6_port),
		 ntohs(actual_addr.sin6_port) != 0);

	close(sk);
}
END_TEST()

FN_TEST(stream_ipv6_dualstack_reject_v4_on_v6only)
{
	int sk = CHECK(socket(AF_INET6, SOCK_STREAM, 0));

	// v6only is 1 by default, so connecting with a bare IPv4 address should fail
	struct sockaddr_in v4_addr = { 0 };
	v4_addr.sin_family = AF_INET;
	v4_addr.sin_port = htons(8080);
	v4_addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

	TEST_ERRNO(connect(sk, (struct sockaddr *)&v4_addr, sizeof(v4_addr)),
		   EAFNOSUPPORT);

	close(sk);
}
END_TEST()
