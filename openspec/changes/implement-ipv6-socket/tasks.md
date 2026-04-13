## 1. Add AF_INET6 Socket Creation Support

- [x] 1.1 Update `kernel/src/syscall/socket.rs` to add AF_INET6 branch in `sys_socket`
  - Add `(CSocketAddrFamily::AF_INET6, SockType::SOCK_STREAM)` case returning `StreamSocket`
  - Add `(CSocketAddrFamily::AF_INET6, SockType::SOCK_DGRAM)` case returning `DatagramSocket`

## 2. Fix IPv6 Bind Support

- [x] 2.1 Update `kernel/src/net/socket/ip/common.rs` to support IPv6 in `get_iface_to_bind`
  - Add IPv6 loopback interface lookup
  - Handle IPv6 addresses in `get_iface_to_bind` and `get_ephemeral_iface`

## 3. Add IPv6 Address to Loopback Interface

- [x] 3.1 Update `kernel/src/net/iface/init.rs` to add IPv6 loopback address `::1`
- [x] 3.2 Add `ipv6_addr()` method to Iface trait in `aster-bigtcp`

## 4. Fix BoundPort IPv6 Endpoint

- [x] 4.1 Update `BoundPort::endpoint()` to return IPv6 address if available

## 5. Verify Implementation

- [x] 5.1 Build the kernel to ensure no compilation errors
- [ ] 5.2 Run IPv6 tests (`tcp6_server`, `tcp6_client`, `udp6_server`, `udp6_client`)
- [ ] 5.3 Verify existing IPv4 tests still pass
