# IPv4 实现详析——分层架构与代码结构

> 更新时间: 2026-04-12
> Asterinas 版本: 0.17.1

---

## 一、整体分层架构

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              应用层 (Userspace)                                      │
│        socket() → bind() → listen() → accept() → connect() → recv()/send()          │
└───────────────────────────────────────────────────────────────────────────────────────┘
                                         ↓↑
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                          Syscall 层 (kernel/src/syscall/)                           │
│  ┌──────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────┐   │
│  │socket.rs │  │ bind.rs │  │listen.rs│  │accept.rs│  │connect.rs│  │send.rs │   │
│  └────┬─────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘   │
│       │             │            │            │            │            │             │
│       └─────────────┴──────────┬┴──────────┴────────────┴────────────┘             │
│                                ↓                                                     │
├───────────────────────────────────────────────────────────────────────────────────────┤
│                      Socket API 层 (kernel/src/net/socket/)                           │
│  ┌─────────────────────────────────────────────────────────────┐                      │
│  │                     Socket trait                          │                       │
│  │  bind() → connect() → listen() → accept() → recvmsg/sendmsg                     │
│  └──────────────────────────┬──────────────────────────────┘                       │
│              ┌─────────────┴─────────────┐                                        │
│              ▼                          ▼                                        │
│  ┌───────────────────┐       ┌───────────────────┐                                │
│  │  StreamSocket     │       │  DatagramSocket   │                                │
│  │  (TCP)           │       │  (UDP)            │                                │
│  │  ├─ init.rs      │       │  ├─ bound.rs     │                                │
│  │  ├─ listen.rs   │       │  │  unbound.rs    │                                │
│  │  ├─ connected.rs│       │  └─ observer.rs  │                                │
│  │  ├─ connecting.rs              │                                                    │
│  │  └─ observer.rs │       ▼                       │                             │
│  │  ─────────────────  │  ┌───────────────────┐  │                             │
│  │  ip/addr.rs ──────▶│  │  ip/common.rs     │◀─┘                             │
│  │  地址转换层       │  │  get_iface_to_bind│                                 │
│  └───────────────────┘  │  bind_port        │                                   │
│                       └───────────────────────────────────────────────────────────────┘  │
│                                    │                                              │
├────────────────────────────────────┴──────────────────────────────────────────────┤
│                  bigtcp 层 (kernel/libs/aster-bigtcp/src/)                         │
│  ┌──────────────────────────────────────────────────────┐                        │
│  │  Socket Table                                          │                       │
│  │  ├─ ListenerHashBucket (TCP 监听器)                  │                       │
│  │  ├─ ConnectionHashBucket (TCP 连接)                   │                       │
│  │  └─ UdpSocket 列表                                   │                       │
│  └──────────────────────────────────────────────────────┘                        │
│                   │                              │                                │
│       ┌──────────┴──────────┐          ┌──────────┴──────────┐                   │
│       ▼                   ▼          ▼                   ▼                       │
│  ┌─────────────┐    ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │TcpListener │    │TcpConnection│  │UdpSocket  │  │BoundPort  │            │
│  │ (server)   │    │ (client)    │  │           │  │           │            │
│  └─────┬─────┘    └─────┬─────┘  └─────┬─────┘  └─────┬─────┘            │
│        │                │               │              │                            │
├────────┴────────────────┴──────────────┴──────────────┴─────────────────────────┤
│                          Interface 层 (aster-bigtcp/src/iface/)                   │
│  ┌��────────────────────────────────────────────────────────────────────────┐      │
│  │                    EtherIface / IpIface                              │      │
│  │  ├─ PollContext::poll_ingress()   入口流处理                         │      │
│  │  │   ├─ parse_and_process_ipv4()  IPv4 头解析                        │      │
│  │  │   ├─ parse_and_process_tcp()  TCP 处理                          │      │
│  │  │   └─ parse_and_process_udp()   UDP 处理                          │      │
│  │  ├─ PollContext::poll_egress()   出口流处理                         │      │
│  │  │   ├─ dispatch_tcp()           TCP 发送调度                       │      │
│  │  │   └─ dispatch_udp()          UDP 发送调度                       │      │
│  │  ├─ Socket Table 操作                                              │      │
│  │  │   ├─ lookup_listener()       查找监听器                         │      │
│  │  │   ├─ lookup_connection()    查找连接                          │      │
│  │  │   └─ insert_*() / remove_*()   管理连接/监听                       │      │
│  │  └─ ARP 处理                                                        │      │
│  │      ├─ process_arp()           ARP 响应                          │      │
│  │      └─ resolve_ether_or_generate_arp() ARP 解析                │      │
│  └─────────────────────────────────────────────────────────────────────────┘      │
│                                    │                                              │
├────────────────────────────────────┴──────────────────────────────────────────────┤
│                        smoltcp Wire 层 (protocol definitions)                    │
│  Ipv4Packet, TcpPacket, UdpPacket, ArpPacket, EthernetFrame                     │
├───────────────────────────────────────────────────────────────────────────────────────┤
│                         驱动层 (aster_virtio / ostd)                            │
│                     VirtioNet, Loopback, Device traits                         │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、代码位置速查表

### 2.1 Syscall 层

| 文件 | 职责 | 入口函数 |
|------|------|----------|
| `syscall/socket.rs` | 创建 socket | `sys_socket(domain, type, protocol)` |
| `syscall/bind.rs` | 绑定地址 | `sys_bind(sockfd, addr, len)` |
| `syscall/connect.rs` | 建立连接 | `sys_connect(sockfd, addr, len)` |
| `syscall/listen.rs` | 监听 | `sys_listen(sockfd, backlog)` |
| `syscall/accept.rs` | 接受连接 | `sys_accept(sockfd)` / `sys_accept4()` |

### 2.2 Socket API 层

| 目录 | 文件 | 职责 |
|------|------|------|
| `socket/ip/` | `addr.rs` | SocketAddr ↔ IpEndpoint 转换 |
| | `common.rs` | iface 选择、端口绑定 |
| | `stream/` | `init.rs` | InitStream (初始状态) |
| | | `listen.rs` | ListenStream (监听状态) |
| | | `connected.rs` | ConnectedStream (已��接) |
| | | `connecting.rs` | ConnectingStream (连接中) |
| | | `options.rs` | TCP 选项 |
| `socket/ip/datagram/` | `mod.rs` | DatagramSocket |
| | `bound.rs` | BoundDatagram (已绑定) |
| | `unbound.rs` | UnboundDatagram (未绑定) |

### 2.3 bigtcp 层

| 目录 | 文件 | 职责 |
|------|------|------|
| `bigtcp/socket/` | `bound/tcp_conn.rs` | TCP 连接状态机 |
| | `bound/tcp_listen.rs` | TCP 监听/backlog |
| | `bound/udp.rs` | UDP socket |
| | `socket_table.rs` | 全局 socket 哈希表 |
| `bigtcp/iface/` | `poll.rs` | 包收发调度 |
| | `phy/ether.rs` | EtherIface (以太网 + ARP) |
| | `common.rs` | 接口公共组件、端口管理 |

---

## 三、TCP 建联流程（客户端）

```
1. syscall/connect.rs
   ↓
2. socket/connect(socket_addr)
   │
   ├─ SocketAddr (用户态) → IpEndpoint (bigtcp)
   │   文件: ip/addr.rs::try_into()
   │
   ▼
3. StreamSocket::connect(socket_addr)
   │
   ▼
4. InitStream::connect()
   │
   ├─ 如果未绑定 → bind_port(随机端口)
   │   ├─ ip/common.rs::get_ephemeral_iface()
   │   │   └─ 根据目标 IP 选择 iface (virtio > loopback)
   │   └─ iface.bind(BindPortConfig)
   │       └─ aster-bigtcp::iface::common.rs::bind_port()
   │           └─ 分配端口 (32768-60999 范围)
   │
   ▼
5. ConnectingStream::new()
   │
   ├─ TcpConnection::new_connect(bound_port, remote_endpoint)
   │   ├─ 创建 RawTcpSocket
   │   ├─ 设置连接 key: (local_ip, local_port, remote_ip, remote_port)
   │   └─ 注册到 socket_table
   │
   ▼
6. 如果同步完成 → ConnectedStream
   │    如果拒绝 → ConnResult::Refused
   │    如果进行中 → ConnectingStream (等待 poll)
   │
   ▼
7. EtherIface::poll_ingress() + poll_egress()
   │
   ├─ TCP 三次握手在 smoltcp 层完成
   │   状态机: Closed → SynSent → Established
   │
   └─ 后续 recv()/send() 通过 ConnectedStream
```

---

## 四、TCP 监听流程（服务器端）

```
1. syscall/listen.rs
   ↓
2. StreamSocket::listen(backlog)
   │
   ├─ 需要先 bind() 到具体端口
   │   (TODO: INADDR_ANY 0.0.0.0 未支持)
   │
   ▼
3. InitStream::listen(backlog)
   │
   ├─ 获取 bound_port
   │
   ▼
4. ListenStream::new()
   │
   ├─ TcpListener::new_listen(bound_port, max_conn)
   │   ├─ 创建 listener_key: (ip, port)
   │   ├─ 检查 socket_table 中是否已存在
   │   │   └─ 如果存在 → ListenError::AddressInUse
   │   │
   │   ├─ 创建 RawTcpSocket
   │   │   └─ socket.listen(local_endpoint)
   │   │
   │   ├─ 注册监听器到 socket_table
   │   │   └─ insert_listener()
   │   │
   │   └─ 创建 TcpBacklog 结构
   │
   ▼
5. StreamSocket::accept()
   │
   ├─ ListenStream::accept()
   │   ├─ 从 backlog.connected 获取连接
   │   └─ 或等待 Syn 消息
   │
   ▼
6. EtherIface::poll_ingress()
   │
   ├─ parse_and_process_tcp()
   │   ├─ 查找 ConnectionKey → 现有连接
   │   │   └─ TcpConnection::process()
   │   │
   │   ├─ 如果没有连接 → 查找 ListenerKey
   │   │   ├─ TcpListener::process()
   │   │   │   ├─ 创建新 TcpConnection
   │   │   │   └─ 返回 SYN-ACK
   │   │   └─ 添加到 backlog.connecting
   │   │
   │   └─ 三次握手完成 → 移动到 connected
   │
   └─ Accept 返回新 StreamSocket
```

---

## 五、UDP 收发流程

### 5.1 发送端 (sendto)

```
1. syscall/sendto.rs
   ↓
2. DatagramSocket::sendmsg(reader, message_header, flags)
   │
   ├─ 解析目标 endpoint
   │
   ▼
3. select_remote_and_bind() / bind_ephemeral()
   │
   ├─ 如果未绑定 → 创建 BoundDatagram
   │   ├─ 分配临时端口
   │   └─ 注册到 iface
   │
   ▼
4. bound_datagram.try_send()
   │
   ├─ 调用 smoltcp RawUdpSocket::dispatch()
   │   └─ 生成 IpRepr + UdpRepr
   │
   ▼
5. EtherIface::dispatch_udp()
   │
   ├─ 解析目标 IP → ARP 解析
   │   ├─ 如果不在 ARP 表中 → 发送 ARP 请求
   │   └─ 等待响应后重试
   │
   ├─ 构造 EthernetFrame + Ipv4Packet + UdpPacket
   │
   └─ 发送到设备
```

### 5.2 接收端 (recvfrom)

```
1. EtherIface::poll_ingress()
   │
   ├─ parse_ip_or_process_arp()
   │   ├─ Ethernet 帧解析
   │   └─ IPv4 解析
   │
   ▼
2. PollContext::parse_and_process_udp()
   │
   ├─ UdpPacket 解析
   │
   ▼
3. PollContext::process_udp()
   │
   ├─ 遍历所有 UDP sockets
   │   ├─ socket.can_process(dst_port)?
   │   └─ socket.process()
   │       └─ 放入 socket 接收缓冲区
   │
   └─ 返回数据
```

### 5.3 数据获取

```
1. DatagramSocket::recvmsg()
   │
   ├─ Inner::try_recv()
   │   └─ 从 BoundDatagram 读取数据
   │
   └─ 返回 (data, remote_addr)
```

---

## 六、地址转换层

```
                    SocketAddr (用户态)
                         │
         ┌───────────────┼───────────────┐
         │               │               │
        IPv4          Unix          Netlink
         │               │               │
         ▼               │               ▼
    ┌──────────┐        │          ┌──────────┐
    │Ipv4Address│        │          │          │
    │   +      │        │          │          │
    │PortNum   │        │          │          │
    └────┬─────┘        │          └──────────┘
         │              │
         └──────────────┼─────────────────────▶ IpEndpoint (bigtcp)
                        │
                        ▼
                 ┌─────────────┐
                 │IpAddress   │
                 │    +      │
                 │Port      │
                 └─────────┘
```

反向转换：

```
                    IpEndpoint (bigtcp)
                         │
         ┌───────────────┴───────────────┐
         ▼                               ▼
    ┌──────────┐                  ┌──────────┐
    │Ipv4      │                  │  IPv6    │ (TODO)
    └────┬─────┘                  └──────────┘
         │
         └─────────────────────▶ SocketAddr (用户态)
```

---

## 七、IPv4 Packet 处理流程

```
                    数据包到达 (poll_ingress)
                         │
                         ▼
    ┌────────────────────────────────────────────┐
    │  EtherIface::parse_ip_or_process_arp()   │
    ├────────────────────────────────────────────┤
    │  1. 以太网帧解析                          │
    │     EthernetFrame::new_checked()         │
    │     EthernetRepr::parse()               │
    │                                         │
    │  2. 检查目标 MAC                         │
    │     - 广播地址 ✓                        │
    │     - 自身 MAC ✓                        │
    │     - 其他 → 丢弃                        │
    │                                         │
    │  3. 协议分发                            │
    │     - Ethertype::Ipv4 → IPv4 处理       │
    │     - Ethertype::Arp → ARP 处理        │
    │     - 其他 → 丢弃                       │
    └────────────────────────────────────────────┘
                         │
             ┌───────────┴───────────┐
             ▼                       ▼
    ┌────────────────┐      ┌────────────────┐
    │   IPv4 处理     │      │   ARP 处理     │
    │                 │      │               │
    │ Ipv4Packet::   │      │ ArpPacket::   │
    │ new_checked()  │      │ new_checked() │
    │                 │      │               │
    │ Ipv4Repr::parse│      │ ArpRepr::parse│
    └───────┬────────┘      └───────┬────────┘
            │                       │
            └───────────┬───────────┘
                        ▼
    ┌────────────────────────────────────────────┐
    │   PollContext::parse_and_process_ipv4()    │
    ├────────────────────────────────────────────┤
    │  1. 验证 IPv4 头                          │
    │     - 版本 = 4                           │
    │     - 头长度 >= 5                        │
    │     - 总长度 >= 头长度                   │
    │                                         │
    │  2. 检查目标地址                         │
    │     - 广播地址 → ICMP Unreachable       │
    │     - 本地单播地址 ✓                    │
    │     - 非本地 → 路由查找                   │
    │                                         │
    │  3. 协议分发                            │
    │     - IpProtocol::Tcp → TCP 处理        │
    │     - IpProtocol::Udp → UDP 处理        │
    │     - Icmp → ICMP 处理 (TODO)         │
    │     - 其他 → 忽略                      │
    └────────────────────────────────────────────┘
```

---

## 八、Socket Table 哈希结构

```
                    Socket Table
    ┌────────────────────────────────────────┐
    │                                        │
    │  [ListenerHashBucket] × 64               │
    │  ├─ 0: Vec<TcpListener>               │
    │  ├─ 1: Vec<TcpListener>               │
    │  └─ ...                             │
    │      (hash = jhash(addr, port)         │
    │              & 63)                    │
    │                                        │
    │  [ConnectionHashBucket] × 8192        │
    │  ├─ 0: Vec<TcpConnection>           │
    │  ├─ 1: Vec<TcpConnection>           │
    │  └─ ...                             │
    │      (hash = jhash(local, remote)      │
    │              & 8191)                  │
    │                                        │
    │  [UdpSocket] × Vec                  │
    │  (线性搜索)                         │
    │                                        │
    └────────────────────────────────────────┘

                        │
                        ▼
              ┌───────────────────┐
              │    查找函数        │
              ├───────────────────┤
              │ lookup_listener() │
              │ lookup_connection│
              │ lookup_udp_socket│
              └───────────────────┘
```

---

## 九、接口初始化

### 9.1 Loopback 接口

```rust
// kernel/src/net/iface/init.rs::new_loopback()
const LOOPBACK_ADDRESS: Ipv4Address = Ipv4Address::new(127, 0, 0, 1);
const LOOPBACK_PREFIX: u8 = 8;  // 255.0.0.0
```

### 9.2 Virtio 网卡接口

```rust
// kernel/src/net/iface/init.rs::new_virtio()
const VIRTIO_ADDRESS: Ipv4Address = Ipv4Address::new(10, 0, 2, 15);
const VIRTIO_PREFIX: u8 = 24;  // 255.255.255.0
const VIRTIO_GATEWAY: Ipv4Address = Ipv4Address::new(10, 0, 2, 2);
```

---

## 十、已知限制

| 限制 | 位置 | 状态 |
|------|------|------|
| INADDR_ANY (0.0.0.0) | `ip/common.rs` | TODO |
| IPv6 支持 | 多处 | TODO |
| 多接口/多 IP | `iface/iface.rs` | TODO |
| ARP 缓存超时 | `phy/ether.rs` | TODO |
| 动态路由 | `phy/ether.rs` | 静态网关 |
| ICMP 响应 | `poll.rs` | 部分支持 |

---

## 十一、关键数据结构

### 11.1 IpEndpoint

```rust
pub struct IpEndpoint {
    pub addr: IpAddress,      // Ipv4Address 或 Ipv6Address (TODO)
    pub port: PortNum,        // 16 位端口号
}
```

### 11.2 ConnectionKey

```rust
pub struct ConnectionKey {
    local_addr: IpAddress,
    local_port: PortNum,
    remote_addr: IpAddress,
    remote_port: PortNum,
    hash: SocketHash,
}
```

### 11.3 ListenerKey

```rust
pub struct ListenerKey {
    addr: IpAddress,
    port: PortNum,
    hash: SocketHash,
}
```

---

## 十二、端口分配范围

```rust
const IP_LOCAL_PORT_START: u16 = 32768;
const IP_LOCAL_PORT_END: u16 = 60999;
```

 Ephemeral 端口范围: 32768 ~ 60999

---

*文档结束*