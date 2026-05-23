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

/// A local endpoint, which indicates that the local endpoint is unspecified.
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
    pub const fn unspecified_endpoint(&self) -> IpEndpoint {
        match self {
            IpAddressFamily::IPv4 => UNSPECIFIED_LOCAL_ENDPOINT,
            IpAddressFamily::IPv6 => UNSPECIFIED_LOCAL_ENDPOINT_V6,
        }
    }
}

/// Returns `true` if the address is an IPv4-mapped IPv6 address (`::ffff:x.x.x.x`).
pub(super) fn is_ipv4_mapped(addr: IpAddress) -> bool {
    if let IpAddress::Ipv6(ipv6) = addr {
        ipv6.to_ipv4_mapped().is_some()
    } else {
        false
    }
}

/// Maps a bare IPv4 endpoint to an IPv4-mapped IPv6 [`SocketAddr`].
///
/// Native IPv6 endpoints pass through unchanged.
// Used by `present_addr` to present dual-stack addresses to the user
// in IPv4-mapped IPv6 form per RFC 4038.
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
// The socket layer normalizes bare IPv4 to IPv4-mapped IPv6 for dual-stack sockets.
// Every dispatch point deeper in the stack must call this function
// to recover the native address for interface lookup,
// broadcast detection, and ephemeral port selection.
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
}

// Note: This does not handle IPv4-mapped IPv6 addresses — it returns `IPv6`
// for them. Callers that need to reject IPv4-mapped addresses when
// `IPV6_V6ONLY` is set must combine this with an explicit `is_ipv4_mapped`
// check (see `prepare_endpoint`).
impl From<IpAddress> for IpAddressFamily {
    fn from(addr: IpAddress) -> Self {
        match addr {
            IpAddress::Ipv4(_) => IpAddressFamily::IPv4,
            IpAddress::Ipv6(_) => IpAddressFamily::IPv6,
        }
    }
}
