## Why

IPv4 support is mature, but IPv6 remains TODO in the current codebase. Enabling IPv6 address IO between user space and kernel space is a prerequisite for supporting IPv6 system calls like bind, connect, and getsockname. This change focuses on establishing the data-plane boundary for IPv6 addresses and does not implement actual IPv6 packet handling.

## What Changes

- Add IPv6 socket address representation at the kernel boundary (CSocketAddrInet6) modeled after the existing IPv4 structure.
- Extend the user/kernel address translation path to support_ipv6:
  - Read: AF_INET6 addresses from user space, map to SocketAddr::IPv6, then to IpEndpoint.
- Extend the write path to emit IPv6 socket addresses back to user space when needed.
- Reuse existing address-serialization/deserialization scaffolding to avoid regressions.
- Update comments to reflect the IPv6 capabilities and boundaries.

## Capabilities

- New: SocketAddr::IPv6(Ipv6Address, PortNum)
- New: AF_INET6 path support for read_socket_addr_from_user and write_socket_addr_with_max_len
- No changes to IPv4 behavior or existing tests beyond added IPv6 support

## Impact

- kernel/src/util/net/addr/ip.rs: New IPv6 structures and conversions
- kernel/src/util/net/addr/family.rs: AF_INET6 handling in read/write paths
- Tests: No new tests added here; will be added alongside IO path tests when available

## Dependencies
- Requires smoltcp IPv6 address types and conversions (already available in the codebase)
- Requires no new external dependencies

## Acceptance Criteria

- AF_INET6 read path reads 16-byte IPv6 address and port and returns SocketAddr::IPv6
- AF_INET6 write path takes SocketAddr::IPv6 and writes to user space with correct endianness
- No regressions for IPv4 path
- Code compiles with the existing toolchain (Rust/Cargo available in CI)

## Milestones
- [ ] Define CSocketAddrInet6 and CInet6Addr
- [ ] Implement conversion to/from Ipv6Address
- [ ] Extend read_socket_addr_from_user to AF_INET6
- [ ] Extend write_socket_addr_with_max_len for IPv6
- [ ] Add minimal tests for conversion paths (if allowed by repo policy)
