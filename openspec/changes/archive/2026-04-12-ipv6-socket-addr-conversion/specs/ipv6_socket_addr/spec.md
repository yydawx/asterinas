## ADDED Requirements

### Requirement: IPv6 Socket Address Support

Asterinas 内核 **SHALL** 支持 IPv6 socket 地址的创建、传输和转换。

#### Scenario: 创建 IPv6 SocketAddr

- **WHEN** 用户构造 `SocketAddr::IPv6(ipv6_addr, port)`
- **THEN** 返回包含 IPv6 地址和端口的 SocketAddr

#### Scenario: SocketAddr 转换为 IpEndpoint (IPv4)

- **WHEN** 将 `SocketAddr::IPv4(ipv4_addr, port)` 转换为 `IpEndpoint`
- **THEN** 返回 `IpEndpoint` 包含对应的 IPv4 地址和端口
- **AND** 地址类型为 `IpAddress::Ipv4`

#### Scenario: SocketAddr 转换为 IpEndpoint (IPv6)

- **WHEN** 将 `SocketAddr::IPv6(ipv6_addr, port)` 转换为 `IpEndpoint`
- **THEN** 返回 `IpEndpoint` 包含对应的 IPv6 地址和端口
- **AND** 地址类型为 `IpAddress::Ipv6`

#### Scenario: IpEndpoint 转换为 SocketAddr (IPv4)

- **WHEN** 将包含 IPv4 地址的 `IpEndpoint` 转换为 `SocketAddr`
- **THEN** 返回 `SocketAddr::IPv4(ipv4_addr, port)`

#### Scenario: IpEndpoint 转换为 SocketAddr (IPv6)

- **WHEN** 将包含 IPv6 地址的 `IpEndpoint` 转换为 `SocketAddr`
- **THEN** 返回 `SocketAddr::IPv6(ipv6_addr, port)`

#### Scenario: 不支持的地址族返回错误

- **WHEN** 尝试将不支持的 `SocketAddr` 变体（如 Unix、Vsock）转换为 `IpEndpoint`
- **THEN** 返回 `Errno::EAFNOSUPPORT` 错误
