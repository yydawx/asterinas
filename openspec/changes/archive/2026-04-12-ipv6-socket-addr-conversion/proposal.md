## 为什么

Asterinas 目前只支持 IPv4，不支持 IPv6 地址。用户尝试使用 IPv6 地址
（如 `connect()`、`bind()`）时会返回 EAFNOSUPPORT 错误。

这限制了Asterinas在现代网络环境中的使用。

## 什么会改变

- SocketAddr 枚举添加 IPv6 变体
- SocketAddr ↔ IpEndpoint 转换函数添加 IPv6 支持

## 新增能力

- `SocketAddr::IPv6` - 支持 IPv6 socket 地址
- IPv6 地址的用户态/内核态转换

## 影响

- `kernel/src/net/socket/util/socket_addr.rs` - 添加 IPv6 变体
- `kernel/src/net/socket/ip/addr.rs` - 添加转换逻辑
