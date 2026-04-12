## ADDED Requirements

### Requirement: IPv6 Address IO Boundaries

Asterinas MUST support IPv6 addresses read from and written to user space, forming the IO boundary between user mode and the kernel network stack. This covers the transformation between user-space sockaddr_in6 equivalents and kernel-side IpEndpoint, without implementing actual IPv6 packet handling.

#### Scenario: Read IPv6 address from user space
- **WHEN** a user space provides an IPv6 sockaddr_in6 structure via read_socket_addr_from_user
- **THEN** the function returns SocketAddr::IPv6(ipv6_addr, port) and the caller can convert to IpEndpoint via TryFrom/From as needed

#### Scenario: Write IPv6 address to user space
- **WHEN** a SocketAddr::IPv6(addr, port) is written via write_socket_addr_with_max_len
- **THEN** the function writes the corresponding sockaddr_in6 representation to user space (respecting max_len) and returns the actual length written

#### Scenario: Not supported address families
- **WHEN** an unsupported address family is requested (e.g., AF_UNSPEC, AF_UNIX, etc.)
- **THEN** read/write paths return EAFNOSUPPORT or invalid length errors as appropriate

### Notes
- The IPv6 IO path is designed to mirror the IPv4 path structure to maintain consistency and ease maintenance.
- The exact C struct layout (CSocketAddrInet6) and conversion helpers are described in Design.md and will be implemented in the code base.

## DELTAS
- ADDED: AF_INET6 handling in read_socket_addr_from_user (CSocketAddrInet6 parsing)
- ADDED: AF_INET6 handling in write_socket_addr_with_max_len (IPv6 write path)
- ADDED: IPv6 address container types (CSocketAddrInet6, CInet6Addr) and conversions (to/from Ipv6Address)
