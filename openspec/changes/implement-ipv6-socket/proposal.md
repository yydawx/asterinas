## Why

Asterinas 内核目前不支持 AF_INET6（IPv6）socket，导致 IPv6 网络测试失败。用户态程序创建 IPv6 socket 时返回 `EAFNOSUPPORT`。这是 Linux 兼容性的基础缺失，IPv6 是现代互联网的事实标准。

## What Changes

- 在 `sys_socket` 系统调用中添加 AF_INET6 socket 创建支持
- 复用现有的 `StreamSocket` 和 `DatagramSocket`，利用 smoltcp 已有的 IPv6 支持
- 不需要修改底层网络栈，smoltcp 原生支持 IPv4/IPv6

## Capabilities

### New Capabilities

- `ipv6-socket-creation`: 支持通过 `socket(AF_INET6, SOCK_STREAM, 0)` 和 `socket(AF_INET6, SOCK_DGRAM, 0)` 创建 IPv6 TCP/UDP socket

### Modified Capabilities

- 无

## Impact

- **代码影响**: `kernel/src/syscall/socket.rs` - 在 match 中添加 AF_INET6 分支
- **API 影响**: 新的系统调用能力，允许用户态创建 IPv6 socket
- **依赖**: smoltcp 已支持 IPv6，无需额外依赖
