## Why

BoundPort::endpoint() 方法当前优先返回 IPv6 地址（如果可用），但底层 socket 可能实际绑定到 IPv4 接口。这导致 ConnectionKey 使用不正确的本地 IP 地址，当尝试查找或管理连接时可能引发问题，特别是在存在 IPv6 和 IPv4 接口的混合环境中。

## What Changes

- 修改 `BoundPort::endpoint()` 方法，使其根据实际绑定的接口地址类型返回相应的 IP 地址（IPv4 或 IPv6）
- 确保 `endpoint()` 方法返回的地址与 socket 实际绑定的接口地址族一致
- 更新相关代码以支持此行为，包括可能的测试修改

## Capabilities

### New Capabilities
- `ipv6-endpoint-fix`: 修复 BoundPort::endpoint() 方法，使其正确返回与实际绑定接口匹配的地址族

### Modified Capabilities
- `ipv6-socket-creation`: 虽然此能力主要关注 socket 创建，但其实现依赖于正确的 endpoint 行为，因此需确保其与修复后的行为兼容

## Impact

- **代码影响**: `kernel/libs/aster-bigtcp/src/iface/common.rs` 中的 `BoundPort::endpoint()` 方法
- **API 影响**: 无公开 API 更改，仅内部行为修正
- **依赖**: 无新依赖
- **测试**: 可能需要更新或添加测试以验证 IPv4 和 IPv6 绑定场景下的正确行为