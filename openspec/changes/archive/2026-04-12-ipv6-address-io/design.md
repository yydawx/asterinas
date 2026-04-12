## Design

背景
- 当前实现已经在用户态 SocketAddr 与内核态 IpEndpoint 之间完成 IPv6 的双向转换，但尚未实现对 IPv6 地址在用户态到内核态 IO 的完整读取/写入路径。
- 目标是在不影响现有 IPv4 流程的前提下，引入 IPv6 的套接字地址 IO 的边界数据结构和读写逻辑，为后续 IPv6 系统调用打好基础。

目标与范围
- 目标：实现 IPv6 地址在用户态和内核态之间的读写 IO 路径，但不实现 IPv6 数据包收发及完整的 IPv6 系统调用实现。
- 范围：仅处理套接字地址（sockaddr_in6 等）的字段解析与序列化，保持与现有 IPv4 路径风格一致，便于维护和对比。

关键决策
- 决策1：新增 IPv6 端边界结构（CSocketAddrInet6）及 IPv6 地址容器（例如 CInet6Addr）以对齐 Linux sockaddr_in6 的字段组织。
- 决策2：在读取路径 read_socket_addr_from_user 中增加 AF_INET6 的分支，读取 IPv6 地址和端口，映射到 SocketAddr::IPv6，再转为 IpEndpoint。
- 决策3：在写入路径 write_socket_addr_with_max_len 中增加 IPv6 的分支，将 SocketAddr::IPv6 转换为对应的 C 结构并写回用户空间。
- 决策4：保持对 IPv4 的现有实现不变，确保回滚在遇到 IPv6 IO 新增时的稳定性。

实现要点
- 字节序：端口号使用网络字节序，IPv6 地址按 16 字节顺序处理；地址族字段遵循 AF_INET6 的约定。
- 复用现有的数据结构与工具：沿用现有的 Storage/CSocketAddrFamily 的读取路径，尽量复用现有的对齐、边界检查与错误码返回逻辑。
- 测试策略：优先实现转换路径的单元测试（如 cfg(test) 模块中的简单转换用例），确保 IPv6 路径在最小路径上可用；集成测试部分留待后续 IPv6 完整 IO 实现就绪再补充。

风险与缓解
- 风险：字节序/对齐错误容易在跨边界 IO 时导致崩溃。缓解：严格参考 IPv4 的实现结构，采用相同的字段布局和边界检查，并在提交前本地编译通过。
- 风险：与现有的测试框架耦合较紧，若添加了未实现的结构，可能影响构建。缓解：在设计阶段对接收边界，避免对未实现部分的推断实现。

验收标准
- AF_INET6 的读取路径能够从用户态读取 IPv6 地址并映射为 SocketAddr::IPv6，然后转换为 IpEndpoint。
- AF_INET6 的写入路径能够将 SocketAddr::IPv6 写回用户态格式，且长度与实现的 max_len 匹配。
- 仍保持 IPv4 的行为完全一致，不影响现有 IPv4 的处理路径。

后续工作
- 实现 CSocketAddrInet6 及相关 From/Into 的转换实现。
- 将 tests 增补到现有测试框架中（在不破坏现有 IPv4 测试的前提下增加 IPv6 的覆盖路径），如未来实现 IPv6 的 IO 时再扩展。
