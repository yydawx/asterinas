## 1. Modify BindPortConfig

- [x] 1.1 Add `addr_family` field to `BindPortConfig` enum variants
- [x] 1.2 Add `endpoint: Option<IpEndpoint>` field to carry IP address information

## 2. Modify IfaceCommon Bind Logic

- [x] 2.1 Update `IfaceCommon::bind()` to pass endpoint info to `BoundPort`
- [x] 2.2 Modify `IfaceCommon::bind_port()` to track address family in `used_ports`

## 3. Modify Port Space Structure

- [x] 3.1 Change `used_ports: BTreeMap<u16, PortState>` to use nested map
- [x] 3.2 Update `alloc_ephemeral_port()` to accept address family parameter
- [x] 3.3 Update port release logic to use address family key

## 4. Modify BoundPort

- [x] 4.1 Add `addr_family: IpAddressFamily` field to `BoundPort<E>` struct
- [x] 4.2 Update `BoundPort` constructor to accept address family
- [x] 4.3 Modify `endpoint()` to return correct IP type based on stored address family

## 5. Update Callers

- [x] 5.1 Update `kernel/src/net/socket/ip/common.rs::bind_port()` to pass endpoint address to `BindPortConfig`
- [x] 5.2 Update all `BindPortConfig` construction sites to include address family

## 6. Add Tests

- [ ] 6.1 Add unit test for IPv4 bound port returning IPv4 endpoint
- [ ] 6.2 Add unit test for IPv6 bound port returning IPv6 endpoint
- [ ] 6.3 Add integration test verifying port space separation
- [ ] 6.4 Add test verifying `ConnectionKey` uses correct address family

---

**Implemented Changes:**

### 1. `port.rs` - BindPortConfig
- Added `IpAddress` field to all variants (CanReuse, Specified, Ephemeral, Backlog)
- Changed `new()` to take `IpEndpoint` instead of `(port, can_reuse)`
- Added `addr()` method to retrieve the IP address

### 2. `common.rs` - IfaceCommon
- Changed `used_ports` from `BTreeMap<u16, PortState>` to `BTreeMap<IpAddress, BTreeMap<u16, PortState>>`
- Updated `alloc_ephemeral_port()` to take address parameter
- Updated `bind_port()` to use per-address port tracking
- Updated `release_port()` to take address parameter

### 3. `common.rs` - BoundPort
- Added `addr: IpAddress` field
- Updated constructor in `IfaceCommon::bind()`
- Added `addr()` method
- Updated `endpoint()` to return correct IP type based on stored address
- Updated `set_can_reuse()` to use per-address port lookup
- Updated `Drop` to pass address to `release_port()`

### 4. `kernel/src/net/socket/ip/common.rs`
- Updated `bind_port()` to pass full endpoint to `BindPortConfig::new()`

### 5. `socket/bound/tcp_listen.rs`
- Updated `BindPortConfig::Backlog` to include address