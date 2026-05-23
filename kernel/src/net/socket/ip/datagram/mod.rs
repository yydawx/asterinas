// SPDX-License-Identifier: MPL-2.0

use core::sync::atomic::{AtomicBool, Ordering};

use aster_bigtcp::wire::IpEndpoint;
use bound::BoundDatagram;
use unbound::{BindOptions, UnboundDatagram};

use super::addr::{IpAddressFamily, SocketFamily, ipv4_to_ipv4_mapped, is_ipv4_mapped};
use crate::{
    events::IoEvents,
    fs::{pseudofs::SockFs, vfs::path::Path},
    net::{
        iface::is_broadcast_endpoint,
        socket::{
            Socket,
            ip::options::{IpOptionSet, Ipv6OptionSet, SetIpLevelOption},
            options::{Error as SocketError, SocketOption, macros::sock_option_mut},
            private::SocketPrivate,
            util::{
                MessageHeader, SendRecvFlags, SocketAddr,
                datagram_common::{Bound, Inner, select_remote_and_bind},
                options::{GetSocketLevelOption, SetSocketLevelOption, SocketOptionSet},
            },
        },
    },
    prelude::*,
    process::signal::{PollHandle, Pollable, Pollee},
    util::{MultiRead, MultiWrite},
};

mod bound;
pub(super) mod observer;
mod unbound;

pub struct DatagramSocket {
    // Lock order: `inner` first, `options` second
    inner: RwMutex<Inner<UnboundDatagram, BoundDatagram>>,
    options: RwLock<OptionSet>,

    family: IpAddressFamily,
    is_nonblocking: AtomicBool,
    pollee: Pollee,
    pseudo_path: Path,
}

#[derive(Clone, Debug)]
struct OptionSet {
    socket: SocketOptionSet,
    ip: IpOptionSet,
    ipv6: Ipv6OptionSet,
    // TODO: UDP option set
}

impl OptionSet {
    fn new() -> Self {
        let socket = SocketOptionSet::new_udp();
        let ip = IpOptionSet::new_udp();
        let ipv6 = Ipv6OptionSet::new();
        OptionSet { socket, ip, ipv6 }
    }
}

impl DatagramSocket {
    pub fn new(is_nonblocking: bool, family: IpAddressFamily) -> Arc<Self> {
        let unbound_datagram = UnboundDatagram::new();
        Arc::new(Self {
            inner: RwMutex::new(Inner::Unbound(unbound_datagram)),
            options: RwLock::new(OptionSet::new()),
            family,
            is_nonblocking: AtomicBool::new(is_nonblocking),
            pollee: Pollee::new(),
            pseudo_path: SockFs::new_path(),
        })
    }

    /// Normalizes and validates the endpoint for the socket's address family.
    /// Returns `Err` if the endpoint is incompatible with the socket.
    fn prepare_endpoint(&self, endpoint: IpEndpoint) -> Result<IpEndpoint> {
        let v6only = if self.family == IpAddressFamily::IPv6 {
            self.options.read().ipv6.v6only()
        } else {
            false
        };
        let sf = SocketFamily::with_v6only(self.family, v6only);
        let endpoint = sf.normalize_endpoint(endpoint);

        if IpAddressFamily::from(endpoint.addr) != self.family
            || (is_ipv4_mapped(endpoint.addr) && v6only)
        {
            return_errno_with_message!(
                Errno::EAFNOSUPPORT,
                "the protocol family does not match the address family"
            );
        }

        Ok(endpoint)
    }

    fn try_recv(
        &self,
        writer: &mut dyn MultiWrite,
        flags: SendRecvFlags,
    ) -> Result<(usize, SocketAddr)> {
        let (len, remote_endpoint) = self.inner.read().try_recv(writer, flags)?;
        let addr = self.present_addr(remote_endpoint);
        self.pollee.invalidate();
        Ok((len, addr))
    }

    fn try_send(
        &self,
        reader: &mut dyn MultiRead,
        remote: Option<&IpEndpoint>,
        flags: SendRecvFlags,
    ) -> Result<usize> {
        let (sent_bytes, iface_to_poll) = select_remote_and_bind(
            &self.inner,
            remote,
            || {
                let remote_endpoint = remote.ok_or_else(|| {
                    Error::with_message(
                        Errno::EDESTADDRREQ,
                        "the destination address is not specified",
                    )
                })?;
                self.inner
                    .write()
                    .bind_ephemeral(remote_endpoint, &self.pollee)
            },
            |bound_datagram, remote_endpoint| {
                let sent_bytes = bound_datagram.try_send(reader, remote_endpoint, flags)?;
                let iface_to_poll = bound_datagram.iface().clone();
                Ok((sent_bytes, iface_to_poll))
            },
        )?;

        self.pollee.invalidate();
        iface_to_poll.poll();

        Ok(sent_bytes)
    }

    /// Presents a stored endpoint to the user per RFC 4038.
    ///
    /// For AF_INET6 sockets, bare IPv4 addresses are mapped to IPv4-mapped IPv6.
    /// `IPV6_V6ONLY` is not consulted
    /// because the presentation is fixed at bind/connect time.
    fn present_addr(&self, endpoint: IpEndpoint) -> SocketAddr {
        if self.family == IpAddressFamily::IPv6 {
            return ipv4_to_ipv4_mapped(endpoint);
        }
        SocketAddr::from(endpoint)
    }
}

impl Pollable for DatagramSocket {
    fn poll(&self, mask: IoEvents, poller: Option<&mut PollHandle>) -> IoEvents {
        self.pollee
            .poll_with(mask, poller, || self.inner.read().check_io_events())
    }
}

impl SocketPrivate for DatagramSocket {
    fn is_nonblocking(&self) -> bool {
        self.is_nonblocking.load(Ordering::Relaxed)
    }

    fn set_nonblocking(&self, is_nonblocking: bool) {
        self.is_nonblocking.store(is_nonblocking, Ordering::Relaxed);
    }
}

impl Socket for DatagramSocket {
    fn bind(&self, socket_addr: SocketAddr) -> Result<()> {
        let endpoint: IpEndpoint = socket_addr.try_into()?;
        let endpoint = self.prepare_endpoint(endpoint)?;
        let can_reuse = self.options.read().socket.reuse_addr();

        self.inner
            .write()
            .bind(&endpoint, &self.pollee, BindOptions { can_reuse })
    }

    fn connect(&self, socket_addr: SocketAddr) -> Result<()> {
        let endpoint: IpEndpoint = socket_addr.try_into()?;
        let endpoint = self.prepare_endpoint(endpoint)?;
        let can_broadcast = self.options.read().socket.broadcast();
        if !can_broadcast && is_broadcast_endpoint(&endpoint) {
            return_errno_with_message!(
                Errno::EACCES,
                "connecting to a broadcast address without SO_BROADCAST is not allowed"
            );
        }

        self.inner.write().connect(&endpoint, &self.pollee)
    }

    fn addr(&self) -> Result<SocketAddr> {
        let endpoint = self
            .inner
            .read()
            .addr()
            .unwrap_or(self.family.unspecified_endpoint());

        Ok(self.present_addr(endpoint))
    }

    fn peer_addr(&self) -> Result<SocketAddr> {
        let endpoint =
            *self.inner.read().peer_addr().ok_or_else(|| {
                Error::with_message(Errno::ENOTCONN, "the socket is not connected")
            })?;

        Ok(self.present_addr(endpoint))
    }

    fn sendmsg(
        &self,
        reader: &mut dyn MultiRead,
        message_header: MessageHeader,
        flags: SendRecvFlags,
    ) -> Result<usize> {
        // TODO: Deal with flags
        if !flags.is_all_supported() {
            warn!("unsupported flags: {:?}", flags);
        }

        let MessageHeader {
            addr,
            control_messages,
        } = message_header;

        let endpoint: Option<IpEndpoint> = match addr {
            Some(addr) => Some(self.prepare_endpoint(addr.try_into()?)?),
            None => None,
        };

        if let Some(endpoint) = endpoint.as_ref() {
            let can_broadcast = self.options.read().socket.broadcast();
            if !can_broadcast && is_broadcast_endpoint(endpoint) {
                return_errno_with_message!(
                    Errno::EACCES,
                    "sending to a broadcast address without SO_BROADCAST is not allowed"
                );
            }
        }

        if !control_messages.is_empty() {
            // TODO: Support sending control message
            warn!("sending control message is not supported");
        }

        // TODO: Block if the send buffer is full
        self.try_send(reader, endpoint.as_ref(), flags)
    }

    fn recvmsg(
        &self,
        writer: &mut dyn MultiWrite,
        flags: SendRecvFlags,
    ) -> Result<(usize, MessageHeader)> {
        // TODO: Deal with flags
        if !flags.is_all_supported() {
            warn!("unsupported flags: {:?}", flags);
        }

        let (received_bytes, peer_addr) =
            self.block_on(IoEvents::IN, || self.try_recv(writer, flags))?;

        // TODO: Receive control message

        let message_header = MessageHeader::new(Some(peer_addr), Vec::new());

        Ok((received_bytes, message_header))
    }

    fn get_option(&self, option: &mut dyn SocketOption) -> Result<()> {
        sock_option_mut!(match option {
            socket_errors @ SocketError => {
                // TODO: Support socket errors for UDP sockets
                socket_errors.set(None);
                return Ok(());
            }
            _ => (),
        });

        let inner = self.inner.read();
        let options = self.options.read();

        // Deal with socket-level options
        match options.socket.get_option(option, &*inner) {
            Err(err) if err.error() == Errno::ENOPROTOOPT => (),
            res => return res,
        }

        // Deal with IP-level options
        match options.ip.get_option(option) {
            Err(err) if err.error() == Errno::ENOPROTOOPT => (),
            res => return res,
        }

        // Deal with IPv6-level options (only for AF_INET6 sockets)
        if self.family == IpAddressFamily::IPv6 {
            options.ipv6.get_option(option)
        } else {
            return_errno_with_message!(Errno::ENOPROTOOPT, "the socket option is unknown")
        }
    }

    fn set_option(&self, option: &dyn SocketOption) -> Result<()> {
        let inner = self.inner.read();
        let mut options = self.options.write();

        // Deal with socket-level options
        let need_iface_poll = match options.socket.set_option(option, &*inner) {
            Err(err) if err.error() == Errno::ENOPROTOOPT => {
                // Deal with IP-level options
                match options.ip.set_option(option, &*inner) {
                    Err(err) if err.error() == Errno::ENOPROTOOPT => {
                        // Deal with IPv6-level options (only for AF_INET6 sockets)
                        if self.family == IpAddressFamily::IPv6 {
                            options.ipv6.set_option(option)?
                        } else {
                            return Err(err);
                        }
                    }
                    Err(err) => return Err(err),
                    Ok(need_iface_poll) => need_iface_poll,
                }
            }
            Err(err) => return Err(err),
            Ok(need_iface_poll) => need_iface_poll,
        };

        let iface_to_poll = need_iface_poll
            .then(|| match &*inner {
                Inner::Unbound(_) => None,
                Inner::Bound(bound_datagram) => Some(bound_datagram.iface().clone()),
            })
            .flatten();

        drop(inner);
        drop(options);

        if let Some(iface) = iface_to_poll {
            iface.poll();
        }

        Ok(())
    }

    fn pseudo_path(&self) -> &Path {
        &self.pseudo_path
    }
}

impl GetSocketLevelOption for Inner<UnboundDatagram, BoundDatagram> {
    fn is_listening(&self) -> bool {
        false
    }
}

impl SetSocketLevelOption for Inner<UnboundDatagram, BoundDatagram> {
    fn set_reuse_addr(&self, reuse_addr: bool) {
        let Inner::Bound(bound) = self else {
            return;
        };

        bound.bound_port().set_can_reuse(reuse_addr);
    }
}

impl SetIpLevelOption for Inner<UnboundDatagram, BoundDatagram> {
    fn set_hdrincl(&self, _hdrincl: bool) -> Result<()> {
        return_errno_with_message!(
            Errno::ENOPROTOOPT,
            "IP_HDRINCL cannot be set on UDP sockets"
        );
    }
}
