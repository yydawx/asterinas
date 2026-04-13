# Proposal: Fix IPv6 TCP Connection Hang

## Why

IPv6 TCP 连接在调用 `connect()` 时卡住，无法完成三次握手。使用 IPv6 loopback 地址 (`::1`) 进行 TCP 连接时，连接请求被阻塞并返回 `EAGAIN`，但之后连接状态永远无法从 `Connecting` 转变为 `Connected`。相比之下，IPv4 loopback 连接 (`127.0.0.1`) 可以正常工作。

这表明 asterinas 的网络栈在处理 IPv6 TCP 连接时存在问题，可能与 smoltcp 对 IPv6 的支持或 loopback 接口的 IPv6 配置有关。

## What Changes

- 调查并修复 IPv6 TCP 连接的阻塞问题
- 确保 TCP 三次握手能够在 IPv6 loopback 上正常完成
- 可能需要修复以下组件：
  - `aster-bigtcp` 中 `socket.connect()` 对 IPv6 的处理
  - loopback 接口的 IPv6 支持
  - TCP 连接状态机的 IPv6 处理

## Capabilities

### New Capabilities

- `ipv6-tcp-connect`: IPv6 TCP 连接能够成功完成三次握手并建立连接

### Modified Capabilities

- 无现有 spec 需要修改（这是新功能的问题修复）

## Impact

- **受影响组件**：
  - `kernel/libs/aster-bigtcp` - TCP socket 连接实现
  - `kernel/src/net/socket/ip/stream/` - IP stream socket
  - `kernel/src/net/iface/` - 网络接口初始化

---

## 调查发现（供设计阶段参考）

### 问题现象

```
IPv4 (成功):
[    24.673] ConnectingStream::new: TcpConnection created successfully
→ 直接成功，无需等待

IPv6 (卡住):
[    85.062] start_connect: created ConnectingStream successfully
[    85.063] connect: calling wait_events for connection to complete
[    85.064] check_connect: checking connection state
[    85.065] check_connect: state is Connecting, returning EAGAIN
→ 永远阻塞
```

### 关键差异

在 `start_connect` 函数中：

- **IPv4**: `result_or_block = None` → 同步完成，立即返回成功
- **IPv6**: `result_or_block = Some(EINPROGRESS)` → 需要异步等待

这说明 `smoltcp::socket::connect()` 对 IPv4 和 IPv6 的处理方式不同。

### 可能的根因

1. **smoltcp 的 `socket.connect()`** 对 IPv6 可能无法同步完成连接
2. **loopback 接口** 可能不支持 IPv6 数据包的发送/接收
3. **Medium::Ip** 配置可能只启用 IPv4，未启用 IPv6