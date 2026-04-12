## 1. 在 wire.rs 中添加 Ipv6Address 导出

- [x] 1.1 打开 `kernel/libs/aster-bigtcp/src/wire.rs`
- [x] 1.2 在 pub use 中添加 `Ipv6Address`
- [x] 1.3 保存文件

## 2. 在 socket_addr.rs 中添加 IPv6 变体

- [x] 2.1 打开 `kernel/src/net/socket/util/socket_addr.rs`
- [x] 2.2 添加 `use aster_bigtcp::wire::Ipv6Address;` 导入
- [x] 2.3 在 `SocketAddr` 枚举中添加 `IPv6(Ipv6Address, PortNum)` 变体
- [x] 2.4 保存文件

## 3. 在 addr.rs 中添加转换逻辑

- [x] 3.1 打开 `kernel/src/net/socket/ip/addr.rs`
- [x] 3.2 在 `TryFrom<SocketAddr> for IpEndpoint` 实现中添加 IPv6 分支
- [x] 3.3 在 `From<IpEndpoint> for SocketAddr` 实现中添加 IPv6 分支
- [x] 3.4 保存文件

## 4. 验证

- [ ] 4.1 运行 `cargo build --package aster-bigtcp` 确认编译通过
- [ ] 4.2 运行 `cargo build --package kernel` 确认编译通过
- [ ] 4.3 检查是否有 clippy 警告

> 注意：当前环境没有 Rust 工具链，请在有工具链的环境中验证
