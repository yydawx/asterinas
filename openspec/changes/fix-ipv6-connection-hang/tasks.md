# Tasks: Fix IPv6 TCP Connection Hang

## 1. Investigation

- [ ] 1.1 Add detailed logging to `socket.connect()` in aster-bigtcp to trace IPv6 vs IPv4 behavior
- [ ] 1.2 Add logging to `Interface::poll()` to confirm loopback poll is being invoked
- [ ] 1.3 Add logging to `TcpConnection::connect_state()` to track state transitions
- [ ] 1.4 Analyze why IPv4 completes synchronously but IPv6 returns EINPROGRESS

## 2. Root Cause Analysis

- [ ] 2.1 Investigate smoltcp's `socket.connect()` implementation for IPv6
- [ ] 2.2 Check loopback interface Medium configuration for IPv6
- [ ] 2.3 Verify TCP SYN is being sent for IPv6 connections
- [ ] 2.4 Verify TCP SYN-ACK is being received for IPv6 connections

## 3. Fix Implementation

- [ ] 3.1 Implement fix based on root cause analysis
- [ ] 3.2 If smoltcp issue: Add IPv6-specific handling or workaround
- [ ] 3.3 If loopback issue: Fix loopback IPv6 packet handling
- [ ] 3.4 Ensure wait_events properly waits for IPv6 connections

## 4. Verification

- [ ] 4.1 Run tcp6_server and tcp6_client test
- [ ] 4.2 Verify IPv6 TCP connection completes successfully
- [ ] 4.3 Verify message exchange works correctly
- [ ] 4.4 Run full network test suite to ensure no regressions

## Investigation Notes from Exploration

### Key Files to Investigate

1. `kernel/libs/aster-bigtcp/src/socket/bound/tcp_conn.rs`
   - Line 318: `socket.connect()` call
   - Check return value for IPv6 vs IPv4

2. `kernel/libs/aster-bigtcp/src/iface/common.rs`
   - `poll()` method implementation
   - Check if loopback is being polled

3. `kernel/src/net/iface/init.rs`
   - Loopback creation: `new_loopback()`
   - Check Medium configuration

4. `kernel/src/net/socket/ip/stream/mod.rs`
   - `start_connect()` function
   - Line 260-268: Return value logic

### Key Question

**Why does IPv4 return None but IPv6 return Some(EINPROGRESS)?**

Search in smoltcp for:
- `socket.connect()` implementation
- Difference between IPv4 and IPv6 TCP connection