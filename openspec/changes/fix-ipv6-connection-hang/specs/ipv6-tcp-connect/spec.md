# Specification: IPv6 TCP Connect

## ADDED Requirements

### Requirement: IPv6 TCP connection SHOULD complete three-way handshake

IPv6 TCP connections on loopback interface (`::1`) SHALL be able to complete the TCP three-way handshake and transition to connected state within a reasonable timeout period.

#### Scenario: IPv6 TCP client connects to IPv6 server on loopback

- **WHEN** a TCP client socket connects to an IPv6 server listening on loopback address `::1`
- **THEN** the connection SHOULD complete with both peers in ESTABLISHED state
- **AND** subsequent read/write operations SHOULD work correctly

#### Scenario: IPv6 TCP connection completes synchronously for loopback

- **WHEN** a TCP client socket calls `connect()` to a loopback IPv6 address
- **THEN** the connection MAY complete synchronously (without returning EINPROGRESS)
- **OR** if it returns EINPROGRESS, the connection MUST complete within 5 seconds

### Requirement: IPv6 TCP connection state SHOULD transition to Connected

After calling `connect()`, the socket's state SHOULD transition from `Connecting` to `Connected` when the three-way handshake completes.

#### Scenario: check_connect returns connected state

- **WHEN** a client calls `connect()` and then invokes check_connect or waits on IO events
- **AND** the server has accepted the connection (sent SYN-ACK)
- **THEN** the socket state SHOULD be `Connected`
- **AND** further wait calls SHOULD return success immediately

## Implementation Notes

### Current Problem

The IPv6 TCP connection is stuck in `Connecting` state because:

1. `smoltcp::socket::connect()` returns early for IPv6 with `EINPROGRESS`
2. `wait_events()` is called but the connection never completes
3. This suggests either:
   - SYN packet is not being sent/received on loopback
   - poll() is not being invoked for the loopback interface
   - TCP state machine is not progressing

### Expected Test Behavior

```c
// tcp6_client.c should succeed
int sock = socket(AF_INET6, SOCK_STREAM, 0);
connect(sock, (struct sockaddr *)&serv_addr, sizeof(serv_addr));  // Should NOT hang
// If it returns, should be 0 (success)
send(sock, "Hello", 5, 0);  // Should work
```