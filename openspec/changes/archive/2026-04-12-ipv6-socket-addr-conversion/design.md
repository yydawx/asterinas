## 背景

Asterinas 当前使用 `SocketAddr` 枚举表示用户态 socket 地址，
使用 `IpEndpoint` (来自 smoltcp) 表示内核网络栈地址。

两者之间需要互相转换，目前只支持 IPv4。

## 目标 / 非目标

**目标：**
- 添加 `SocketAddr::IPv6` 变体
- 支持 IPv6 地址在用户态和内核态之间转换

**非目标：**
- 不实现 IPv6 包的实际收发（后续任务）
- 不实现 IPv6 的 socket 操作（bind/connect/listen）

## 决策

### 决策 1: 在 SocketAddr 枚举中添加 IPv6

```rust
pub enum SocketAddr {
    IPv4(Ipv4Address, PortNum),
    IPv6(Ipv6Address, PortNum),  // 新增
    // 其他变体...
}
```

**理由：** 与现有 IPv4 变体保持一致的结构，使用 smoltcp 的 Ipv6Address。

### 决策 2: 在 wire.rs 中 re-export Ipv6Address

当前 `aster-bigtcp/src/wire.rs` 只 re-export 了 Ipv4Address。

```rust
pub use smoltcp::wire::{
    EthernetAddress,
    IpAddress,
    IpCidr,
    IpEndpoint,
    Ipv4Address,
    Ipv4Cidr,       // 已有
    Ipv6Address,    // 需要新增
};
```

**理由：** 仅需要 Ipv6Address 用于 SocketAddr 转换。

### 决策 3: 在 addr.rs 中添加转换逻辑

在 `TryFrom<SocketAddr> for IpEndpoint` 和 `From<IpEndpoint> for SocketAddr` 中添加 IPv6 分支。

**理由：** 保持现有的错误处理逻辑不变，只扩展支持范围。

### 决策 4: 保持向后兼容

现有的 IPv4 转换逻辑保持不变，只添加 IPv6 分支，不影响现有功能。
