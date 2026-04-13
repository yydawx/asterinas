## ADDED Requirements

### Requirement: IPv4 and IPv6 Port Space Separation

IPv4 and IPv6 sockets SHALL use independent port spaces. A port number used by an IPv4 socket SHALL NOT conflict with the same port number used by an IPv6 socket.

#### Scenario: IPv4 Bind Does Not Block IPv6 Port

- **WHEN** an IPv4 socket binds to port 8080 on an interface
- **THEN** an IPv6 socket CAN bind to port 8080 on the same interface

#### Scenario: IPv6 Bind Does Not Block IPv4 Port

- **WHEN** an IPv6 socket binds to port 8080 on an interface
- **THEN** an IPv4 socket CAN bind to port 8080 on the same interface

### Requirement: BoundPort Returns Correct IP Address Family

`BoundPort::endpoint()` SHALL return an IP endpoint with the same address family as the one used during binding.

#### Scenario: IPv4 Bound Port Returns IPv4 Endpoint

- **WHEN** a socket binds to an IPv4 address and port
- **THEN** `BoundPort::endpoint()` returns an `IpEndpoint` with `IpAddress::Ipv4`

#### Scenario: IPv6 Bound Port Returns IPv6 Endpoint

- **WHEN** a socket binds to an IPv6 address and port
- **THEN** `BoundPort::endpoint()` returns an `IpEndpoint` with `IpAddress::Ipv6`

#### Scenario: Ephemeral Port Allocation Is Per-Address-Family

- **WHEN** an application requests an ephemeral port for an IPv4 socket
- **THEN** the allocated port is from the IPv4 ephemeral range (32768-60999)
- **AND** does not consider IPv6 port usage

- **WHEN** an application requests an ephemeral port for an IPv6 socket
- **THEN** the allocated port is from the IPv6 ephemeral range (32768-60999)
- **AND** does not consider IPv4 port usage

### Requirement: ConnectionKey Uses Correct Local Address Family

`ConnectionKey` SHALL use the correct local IP address family when identifying a connection.

#### Scenario: IPv4 Connection Has IPv4 Local Address in Key

- **WHEN** a TCP connection is established with an IPv4 local endpoint
- **THEN** the `ConnectionKey.local_addr` field contains an `IpAddress::Ipv4`

#### Scenario: IPv6 Connection Has IPv6 Local Address in Key

- **WHEN** a TCP connection is established with an IPv6 local endpoint
- **THEN** the `ConnectionKey.local_addr` field contains an `IpAddress::Ipv6`