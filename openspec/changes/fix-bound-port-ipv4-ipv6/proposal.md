## Why

`BoundPort::endpoint()` 当前的实现存在 IPv4/IPv6 协议族混淆问题。当 socket 绑定到 IPv4 地址时，`endpoint()` 可能返回 IPv6 地址作为本地端点，导致 `ConnectionKey` 使用错误的 IP 类型。这会导致：

1. **连接建立失败** - 当远程连接使用 IPv4，但本地 endpoint 被报告为 IPv6 时，连接查找可能失败
2. **设计不对称** - 先检查 IPv6 再检查 IPv4 的逻辑不符合网络协议族的正常优先级
3. **端口复用错误** - `used_ports` 是 per-iface 的，但 IPv4 和 IPv6 共享端口空间可能导致冲突

## What Changes

- 修改 `BoundPort::endpoint()` 使其根据实际绑定的 socket 地址族返回正确的本地端点
- 移除 IPv6 优先的不对称逻辑，改为基于绑定时的实际协议族
- 确保 IPv4 和 IPv6 端口空间正确分离
- 在 `ConnectionKey` 创建时使用正确的 IP 类型

## Capabilities

### New Capabilities

- **ipv4-ipv6-port-separation**: 明确区分 IPv4 和 IPv6 的端口空间，每个协议族独立追踪可用端口

### Modified Capabilities

- 无 - 这是实现修复，不改变现有规格要求

## Impact

- 受影响代码：`kernel/src/net/socket/mod.rs` 中的 `BoundPort` 实现
- 相关模块：`ConnectionKey`、socket 绑定逻辑、端口分配
- 测试影响：需要添加测试验证 IPv4 socket 返回 IPv4 endpoint