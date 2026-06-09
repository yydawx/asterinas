// SPDX-License-Identifier: MPL-2.0

use aster_bigtcp::wire::{IpAddress, IpEndpoint, Ipv4Address, Ipv6Address};

use crate::{net::socket::util::SocketAddr, prelude::*};

impl TryFrom<SocketAddr> for IpEndpoint {
    type Error = Error;

    fn try_from(value: SocketAddr) -> Result<Self> {
        match value {
            SocketAddr::IPv4(addr, port) => Ok(IpEndpoint::new(addr.into(), port)),
            SocketAddr::IPv6(addr, port) => Ok(IpEndpoint::new(addr.into(), port)),
            _ => return_errno_with_message!(
                Errno::EAFNOSUPPORT,
                "the address is in an unsupported address family"
            ),
        }
    }
}

impl From<IpEndpoint> for SocketAddr {
    fn from(endpoint: IpEndpoint) -> Self {
        let port = endpoint.port;
        match endpoint.addr {
            IpAddress::Ipv4(addr) => SocketAddr::IPv4(addr, port),
            IpAddress::Ipv6(addr) => SocketAddr::IPv6(addr, port),
        }
    }
}

/// An IPv4 local endpoint, which indicates that the local endpoint is unspecified.
///
/// According to the Linux man pages and the Linux implementation, `getsockname()` will _not_ fail
/// even if the socket is unbound. Instead, it will return an unspecified socket address. This
/// unspecified endpoint helps with that.
pub(super) const UNSPECIFIED_LOCAL_ENDPOINT: IpEndpoint =
    IpEndpoint::new(IpAddress::Ipv4(Ipv4Address::UNSPECIFIED), 0);

/// An IPv6 local endpoint, which indicates that the local endpoint is unspecified.
pub(super) const UNSPECIFIED_LOCAL_ENDPOINT_V6: IpEndpoint =
    IpEndpoint::new(IpAddress::Ipv6(Ipv6Address::UNSPECIFIED), 0);

/// Address family for IP sockets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpAddressFamily {
    IPv4,
    IPv6,
}

impl IpAddressFamily {
    /// Returns the unspecified endpoint for this address family.
    pub(super) const fn unspecified_endpoint(&self) -> IpEndpoint {
        match self {
            IpAddressFamily::IPv4 => UNSPECIFIED_LOCAL_ENDPOINT,
            IpAddressFamily::IPv6 => UNSPECIFIED_LOCAL_ENDPOINT_V6,
        }
    }
}

/// Returns `true` if the address is an IPv4-mapped IPv6 address (`::ffff:x.x.x.x`).
pub(super) fn is_ipv4_mapped(addr: IpAddress) -> bool {
    matches!(addr, IpAddress::Ipv6(v6) if v6 != Ipv6Address::UNSPECIFIED && v6.to_ipv4_mapped().is_some())
}

/// Maps a bare IPv4 endpoint to an IPv4-mapped IPv6 [`SocketAddr`].
///
/// Native IPv6 endpoints pass through unchanged.
// Used by `SocketFamily::present_to_user` to present dual-stack addresses
// to the user in IPv4-mapped IPv6 form per RFC 4038.
pub(super) fn ipv4_to_ipv4_mapped(endpoint: IpEndpoint) -> SocketAddr {
    if let IpAddress::Ipv4(ipv4) = endpoint.addr {
        let mapped = IpAddress::Ipv6(ipv4.to_ipv6_mapped());
        return SocketAddr::from(IpEndpoint::new(mapped, endpoint.port));
    }
    SocketAddr::from(endpoint)
}

/// Returns the embedded IPv4 address if `addr` is an IPv4-mapped IPv6 address (`::ffff:x.x.x.x`).
///
/// Native IPv4, native IPv6, and all other addresses pass through unchanged.
///
/// # Where this must be called
///
/// Any code path that hands an address to the low-level network stack (smoltcp)
/// must call this function, because smoltcp does not understand IPv4-mapped IPv6.
/// Call sites fall into these categories:
///
/// - **Socket send path** — unmap before dispatching to the bound socket
/// - **Socket connect path** — unmap before storing the remote endpoint
/// - **Port binding** — unmap before interface lookup
/// - **Ephemeral endpoint selection** — unmap before choosing a source address
/// - **Broadcast detection** — unmap before checking broadcast address sets
///
/// Idempotency: this function is safe to call on already-unmapped addresses
/// (native IPv4 and native IPv6 pass through unchanged), so callers at different
/// layers can defensively unmap without coordination.
pub(crate) fn unmap_ipv4_addr(addr: IpAddress) -> IpAddress {
    match addr {
        IpAddress::Ipv6(addr) => match addr.to_ipv4_mapped() {
            Some(ipv4) => IpAddress::Ipv4(ipv4),
            None => IpAddress::Ipv6(addr),
        },
        other => other,
    }
}

/// Encapsulates a socket's address family and dual-stack configuration.
///
/// Bundles `IpAddressFamily` with `IPV6_V6ONLY` state so they are always
/// considered together in address normalization and presentation decisions.
#[derive(Clone, Copy, Debug)]
pub(super) struct SocketFamily {
    family: IpAddressFamily,
    v6only: bool,
}

impl SocketFamily {
    /// Creates a new `SocketFamily` with an explicit `IPV6_V6ONLY` value.
    pub fn with_v6only(family: IpAddressFamily, v6only: bool) -> Self {
        Self { family, v6only }
    }

    /// Normalizes an endpoint for this socket's address family.
    ///
    /// In dual-stack mode (IPv6 + !v6only), maps bare IPv4 addresses to
    /// IPv4-mapped IPv6 so the socket layer can process them uniformly.
    pub fn normalize_endpoint(&self, endpoint: IpEndpoint) -> IpEndpoint {
        if self.family == IpAddressFamily::IPv6
            && !self.v6only
            && let IpAddress::Ipv4(ipv4) = endpoint.addr
        {
            return IpEndpoint::new(IpAddress::Ipv6(ipv4.to_ipv6_mapped()), endpoint.port);
        }
        endpoint
    }

    /// Normalizes and validates the endpoint for this socket's address family.
    /// Returns `Err` if the endpoint is incompatible with the socket.
    pub fn validate_endpoint(&self, endpoint: IpEndpoint) -> Result<IpEndpoint> {
        let endpoint = self.normalize_endpoint(endpoint);

        let effective = unmap_ipv4_addr(endpoint.addr);
        let is_unspecified = match effective {
            IpAddress::Ipv4(addr) => addr == Ipv4Address::UNSPECIFIED,
            IpAddress::Ipv6(addr) => addr == Ipv6Address::UNSPECIFIED,
        };
        if !is_unspecified {
            if is_ipv4_mapped(endpoint.addr) && self.v6only {
                return_errno_with_message!(
                    Errno::EAFNOSUPPORT,
                    "IPv4-mapped IPv6 addresses are not allowed when IPV6_V6ONLY is set"
                );
            }

            if IpAddressFamily::from_raw_addr(endpoint.addr) != self.family {
                return_errno_with_message!(
                    Errno::EAFNOSUPPORT,
                    "the protocol family does not match the address family"
                );
            }
        }

        Ok(endpoint)
    }

    /// Prepares an endpoint for bind/connect/sendmsg by normalizing and validating it
    /// against this socket's address family and `IPV6_V6ONLY` setting.
    ///
    /// This is the single entry point for endpoint validation. Both `DatagramSocket`
    /// and `StreamSocket` call this from their respective `prepare_endpoint` methods.
    pub fn prepare_endpoint(
        family: IpAddressFamily,
        v6only: bool,
        endpoint: IpEndpoint,
    ) -> Result<IpEndpoint> {
        Self::with_v6only(family, v6only).validate_endpoint(endpoint)
    }
}
/// Resolves a remote endpoint for connect.
///
/// An unspecified address (`0.0.0.0` or `::`) as a destination is not
/// routable.  On Linux the routing table maps these to the loopback
/// address; here we perform the same resolution explicitly before
/// handing the endpoint to smoltcp.
pub(super) fn resolve_remote_endpoint(family: IpAddressFamily, endpoint: IpEndpoint) -> IpEndpoint {
    let unspecified = match endpoint.addr {
        IpAddress::Ipv4(addr) => addr == Ipv4Address::UNSPECIFIED,
        IpAddress::Ipv6(addr) => addr == Ipv6Address::UNSPECIFIED,
    };
    if unspecified {
        let loopback_addr = match family {
            IpAddressFamily::IPv4 => IpAddress::Ipv4(Ipv4Address::new(127, 0, 0, 1)),
            IpAddressFamily::IPv6 => IpAddress::Ipv6(Ipv6Address::new(0, 0, 0, 0, 0, 0, 0, 1)),
        };
        return IpEndpoint::new(loopback_addr, endpoint.port);
    }
    endpoint
}
impl SocketFamily {
    /// Presents a stored endpoint to the user per RFC 4038.
    ///
    /// For AF_INET6 sockets, bare IPv4 addresses are mapped to IPv4-mapped IPv6 form.
    /// The `IPV6_V6ONLY` setting is intentionally **ignored** — per RFC 4038,
    /// `getsockname`/`getpeername` always present dual-stack addresses in mapped form
    /// regardless of the `IPV6_V6ONLY` socket option.
    pub fn present_to_user(family: IpAddressFamily, endpoint: IpEndpoint) -> SocketAddr {
        if family == IpAddressFamily::IPv6 && matches!(endpoint.addr, IpAddress::Ipv4(_)) {
            return ipv4_to_ipv4_mapped(endpoint);
        }
        SocketAddr::from(endpoint)
    }
}

impl IpAddressFamily {
    /// Returns the raw address family, **without** handling IPv4-mapped IPv6.
    ///
    /// # Footgun
    ///
    /// IPv4-mapped IPv6 addresses (`::ffff:x.x.x.x`) return [`IpAddressFamily::IPv6`],
    /// even though they represent embedded IPv4 addresses. Callers that use this for
    /// routing, binding, or broadcast decisions **must** call [`unmap_ipv4_addr`] first.
    /// For socket-level address validation, use [`SocketFamily::validate_endpoint`]
    /// instead — it combines this check with `IPV6_V6ONLY` handling.
    #[must_use]
    pub fn from_raw_addr(addr: IpAddress) -> Self {
        match addr {
            IpAddress::Ipv4(_) => IpAddressFamily::IPv4,
            IpAddress::Ipv6(_) => IpAddressFamily::IPv6,
        }
    }
}
