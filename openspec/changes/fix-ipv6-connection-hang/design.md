# Design: Fix IPv6 TCP Connection Hang

## Context

### 问题背景

IPv6 TCP 连接在调用 `connect()` 时卡住，无法完成三次握手。从日志分析：

1. Client 创建 IPv6 socket 并调用 `connect(::1, 8080)`
2. 成功创建 `ConnectingStream`
3. 返回 `EINPROGRESS` (需要异步等待)
4. 调用 `wait_events()` 等待连接完成
5. `check_connect()` 始终返回 `EAGAIN`，状态保持 `Connecting`

对比 IPv4：
- IPv4 连接在 `connect()` 时**同步完成**，不需要 `wait_events`

### 关键代码路径

```
Client                                          Server
  │                                              │
  │ ── connect(IPv6) ──────────────────────────▶  │
  │     → start_connect()                         │
  │     → result_or_block = Some(EINPROGRESS) ❌   │
  │     → wait_events() → EAGAIN              │
  │                                              │
  │  (应该: SYN → SYN-ACK → ACK)              │  ::1:8080 监听中
```

## Goals / Non-Goals

### Goals
- 修复 IPv6 TCP 连接卡住的问题
- 使 TCP 三次握手能够在 IPv6 loopback 上正常完成

### Non-Goals
- 不修改 IPv4 TCP 连接（已正常工作）
- 不添加新的网络功能
- 不改动应用程序 API

## Decisions

### D1: 根因位置

**决定**: 问题可能出在 `smoltcp::socket::connect()` 对 IPv6 的处理

**理由**:
- 日志显示 IPv4 和 IPv6 在 `start_connect` 中的行为不同
- IPv4: `result_or_block = None` → 同步完成
- IPv6: `result_or_block = Some(EINPROGRESS)` → 需要等待

**替代方案考虑**:
- A) 在内核层修复连接等待逻辑 → 需要更多代码改动
- B) 在 smoltcp 层修复 IPv6 连接 → 可能需要 upstream 贡献
- C) 在 aster-bigtcp 层添加 IPv6 特殊处理 → 最合适

### D2: Medium 配置

**决定**: 检查 loopback 的 `Medium::Ip` 配置是否同时支持 IPv4 和 IPv6

**当前实现**:
```rust
// init.rs
IpIface::new_with_ipv6(
    Wrapper(Mutex::new(Loopback::new(Medium::Ip))),
    Ipv4Cidr::new(LOOPBACK_ADDRESS, ...),
    Some(Ipv6Cidr::new(LOOPBACK_IPV6_ADDRESS, ...)),
    ...
)
```

**推测**: Medium 应该是统一的，但 smoltcp 内部处理可能有问题。

## Risks / Trade-offs

### R1: smoltcp 依赖

**[风险]**: 问题可能在 smoltcp 库内部，asterinas 无法直接修复

**[缓解措施]**:
1. 先添加日志确认问题位置
2. 如果是 smoltcp 问题，考虑升级版本或添加 workaround

### R2: Loopback IPv6 支持

**[风险]**: loopback 设备对 IPv6 的支持可能不完整

**[缓解措施]**:
1. 添加日志到 `poll_iface` 确认 loopback 是否被正确 poll
2. 检查 virtio 网卡是否能配置 IPv6

### R3: 等待逻辑

**[风险]**: `wait_events` 中的连接等待逻辑可能有问题

**[缓解措施]**:
1. 添加日志确认 `poller.wait()` 是否被唤醒
2. 检查是否有事件通知机制

## 调查计划

### 步骤 1: 确认问题位置

添加日志到以下位置：
1. `socket.connect()` 调用 - 查看返回值
2. `Interface::poll()` 调用 - 确认 poll 是否执行
3. `TcpConnection::connect_state()` - 确认状态变化

### 步骤 2: 检查 Medium 配置

1. 检查 `Loopback` 设备的 Medium 类型
2. 检查 smoltcp 处理 IPv4 vs IPv6 的差异

### 步骤 3: 实现修复

根据调查结果选择修复方案：
- A) 如果是 smoltcp 问题：添加 IPv6 特殊的同步等待逻辑
- B) 如果是 loopback 问题：修复 IPv6 poll 实现
- C) 其他：根据具体原因决定

## Open Questions

1. smoltcp 对 IPv6 的 `socket.connect()` 实现是否有已知问题？
2. 是否需要为 IPv6 配置特殊的网络接口？
3. virtio 网卡是否支持 IPv6？（可能需要配置）