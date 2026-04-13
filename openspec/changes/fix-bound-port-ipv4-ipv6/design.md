## Context

当前 `BoundPort::endpoint()` 的实现只返回 IPv4 地址。问题分析表明：

1. **代码结构**:
   - `BoundPort<E>` 存储 `iface: Arc<dyn Iface<E>>` 和 `port: u16`
   - `endpoint()` 方法调用 `self.iface().ipv4_addr()?` 返回 `Option<Ipv4Address>`
   - `used_ports` 是 `BTreeMap<u16, PortState>`，仅按端口号索引

2. **问题**:
   - 当前实现只返回 IPv4，但根据问题描述，应该存在 IPv6 优先的逻辑
   - 端口空间是 IPv4/IPv6 共享的，未区分协议族
   - `ConnectionKey` 使用传入的 `IpEndpoint`，但没有验证本地地址类型

3. **约束**:
   - 只修改 `aster-bigtcp` crate，不影响 `kernel/` 中的 safe Rust
   - 保持向后兼容，不破坏现有 API

## Goals / Non-Goals

**Goals:**
- 修复 `BoundPort::endpoint()` 返回正确的 IP 类型（IPv4 或 IPv6）
- 明确区分 IPv4 和 IPv6 的端口空间

**Non-Goals:**
- 不修改 `smoltcp` 底层行为
- 不添加新的 socket 选项或 API

## Decisions

### Decision 1: 在 `BoundPort` 中存储协议族信息

**选择**: 在 `BoundPort` 创建时从 `BindPortConfig` 传入的 `IpEndpoint` 记录实际使用的协议族。

**理由**: `endpoint` 是唯一知道绑定地址类型的地方。通过在 bind 时传入 endpoint，可以明确知道协议族。

**备选方案**:
- 方案 A (采用): 在 `BoundPort` 中添加 `addr_family: IpAddressFamily` 字段
- 方案 B: 每次调用 `endpoint()` 时检查 IPv4/IPv6 可用性（性能较差）
- 方案 C: 使用 dynamic dispatch 检查（复杂）

### Decision 2: 修改端口空间结构

**选择**: 将 `used_ports` 从 `BTreeMap<u16, PortState>` 改为 `BTreeMap<IpAddressFamily, BTreeMap<u16, PortState>>`。

**理由**: IPv4 和 IPv6 应该使用独立的端口空间，这是 POSIX 行为。

**备选方案**:
- 方案 A (采用): per-family 端口映射
- 方案 B: 使用 `(family, port)` 复合键（需要修改 key 类型）
- 方案 C: 保持共享端口空间（不符合标准行为）

### Decision 3: 修改 `endpoint()` 返回逻辑

**选择**: `endpoint()` 根据 stored `addr_family` 返回对应协议族的地址。

**理由**: 确保返回的 IP 类型与绑定时使用的类型一致。

## Risks / Trade-offs

**[Risk]**: `BindPortConfig` 不携带地址类型信息
- **Mitigation**: 修改 binder 传入端点而非只传端口号

**[Risk]**: 向后兼容性
- **Mitigation**: 现有代码使用 IPv4 为主，不改变 API 行为，只修复返回值

**[Risk]**: 接口可能同时有 IPv4 和 IPv6 地址
- **Mitigation**: 绑定时显式传入的 endpoint 地址决定协议族