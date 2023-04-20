# 第九周汇报

**致理-信计01  佟海轩 2020012709**

## 本周进展

### 上周遗留问题

- [x] `u32_t sys_now(void)`：获取当前时钟，用于实现定时器。使用 `axhal::time::current_time`

### 上层接口适配

- [ ] `IpAddr`
- [ ] `Ipv4Addr`
- [ ] `SocketAddr`
- [ ] `TcpSocket`
  - [ ] `pub fn new() -> Self`
  - [ ] `pub fn local_addr(&self) -> AxResult<SocketAddr>`
  - [ ] `pub fn peer_addr(&self) -> AxResult<SocketAddr>`
  - [ ] `pub fn connect(&mut self, _addr: SocketAddr) -> AxResult`
  - [ ] `pub fn bind(&mut self, _addr: SocketAddr) -> AxResult`
  - [ ] `pub fn listen(&mut self) -> AxResult`
  - [ ] `pub fn accept(&mut self) -> AxResult<TcpSocket>`
  - [ ] `pub fn shutdown(&self) -> AxResult`
  - [ ] `pub fn recv(&self, _buf: &mut [u8]) -> AxResult<usize>`
  - [ ] `pub fn send(&self, _buf: &[u8]) -> AxResult<usize>`
  - [ ] `fn drop(&mut self) {}`
- [x] `pub(crate) fn init(_net_devs: NetDevices)`



## 下周计划

- 

