## Context

Asterinas 目前仅支持 AF_INET（IPv4）socket，创建 IPv6 socket 时返回 `EAFNOSUPPORT`。已有测试用例 `tcp6_server.c`、`tcp6_client.c` 等因缺少内核支持而失败。

现有的网络栈基于 smoltcp，已经支持 IPv6。之前的变更（ipv6-address-io）已实现了用户态/内核态之间的 IPv6 地址读写。

## Goals / Non-Goals

**Goals:**
- 支持 `socket(AF_INET6, SOCK_STREAM, 0)` 创建 IPv6 TCP socket
- 支持 `socket(AF_INET6, SOCK_DGRAM, 0)` 创建 IPv6 UDP socket

**Non-Goals:**
- 不实现 IPv6 特有的 socket 选项（如 `IPV6_V6ONLY`）
- 不实现完整的 IPv6 网络配置（如地址自动配置）
- 不修改底层网络栈（bigtcp/smoltcp）

## Decisions

**决策 1: 复用现有 socket 类型**

现有 `StreamSocket` 和 `DatagramSocket` 使用 `IpEndpoint`，smoltcp 原生支持 IPv4/IPv6，无需创建新的 socket 类型。

**决策 2: 在 sys_socket 中添加 AF_INET6 分支**

在 `kernel/src/syscall/socket.rs` 的 match 语句中添加：
```rust
(CSocketAddrFamily::AF_INET6, SockType::SOCK_STREAM) => StreamSocket::new(is_nonblocking)
(CSocketAddrFamily::AF_INET6, SockType::SOCK_DGRAM) => DatagramSocket::new(is_nonblocking)
```

**备选方案:**
- 创建独立的 IPv6Socket 类型 → 复杂度高，无必要
- 在现有 socket 创建时自动检测地址类型 → 需要修改更多代码

## Risks / Trade-offs

**风险 1: IPv6 数据包处理可能有问题**

smoltcp 已支持 IPv6，但实际网络通信可能存在边界情况（如 fragment、extension headers）。

→  Mitigation: 先支持 socket 创建，后续通过测试发现并修复问题。

**风险 2: socket 选项不区分 IPv4/IPv6**

当前实现中，某些 IP 层选项可能对 IPv6 不适用。

→  Mitigation: 后续添加 IPv6 特有的 socket 选项支持。

## Open Questions

1. 是否需要实现 `IPV6_V6ONLY` 选项（默认应为 true）？
2. IPv6 的默认端口绑定行为是否与 IPv4 一致？
