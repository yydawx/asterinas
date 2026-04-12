## Tasks

Here are the concrete tasks to implement IPv6 address IO between user space and the kernel space, aligned with IPv4 patterns and the new design.

---

## 1. Define IPv6 socket address structure (CSocketAddrInet6)
- [x] 1.1 Add CSocketAddrInet6 struct in `kernel/src/util/net/addr/ip.rs`.
- [x] 1.2 Add CInet6Addr type to represent a 16-byte IPv6 address.
- [x] 1.3 Implement From<(Ipv6Address, PortNum)> for CSocketAddrInet6.
- [x] 1.4 Implement From<CSocketAddrInet6> for (Ipv6Address, PortNum).

---

## 2. Extend read path to AF_INET6
- [x] 2.1 Update `kernel/src/util/net/addr/family.rs` to include AF_INET6 in read_socket_addr_from_user.
- [x] 2.2 Validate length and parse via CSocketAddrInet6.
- [x] 2.3 Map to SocketAddr::IPv6 and return.

---

## 3. Extend write path to IPv6
- [x] 3.1 Update `kernel/src/util/net/addr/family.rs` to include SocketAddr::IPv6 in write_socket_addr_with_max_len.
- [x] 3.2 Convert SocketAddr::IPv6(addr, port) to CSocketAddrInet6 and write to user space.
- [x] 3.3 Ensure correct handling of max_len and actual length reporting.

---

## 4. Validation
- [x] 4.1 Build with current toolchain: ensure no regressions for IPv4 path.
- [x] 4.2 Run existing IPv4 tests for compatibility (no IPv6 tests added yet).
- [x] 4.3 Prepare minimal unit tests for conversion paths if allowed by CI (cfg(test) blocks).

---

## Milestones
- [x] M1: IPv6 structures defined
- [x] M2: AF_INET6 read path implemented
- [x] M3: AF_INET6 write path implemented
- [x] M4: Basic validation completed
- [x] M5: Archive/record and update documentation as needed
