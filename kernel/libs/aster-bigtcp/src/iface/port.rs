// SPDX-License-Identifier: MPL-2.0

use smoltcp::wire::{IpAddress, IpEndpoint};

/// The configuration using for bind to a TCP/UDP port.
pub enum BindPortConfig {
    /// Binds to the specified non-reusable port.
    CanReuse(IpAddress, u16),
    /// Binds to the specified reusable port.
    Specified(IpAddress, u16),
    /// Allocates an ephemeral port to bind.
    Ephemeral(IpAddress, bool),
    /// Reuses the port of the listening socket.
    Backlog(IpAddress, u16),
}

impl BindPortConfig {
    /// Creates new configuration using for bind to a TCP/UDP port.
    pub fn new(endpoint: IpEndpoint, can_reuse: bool) -> Self {
        let port = endpoint.port;
        match (port, can_reuse) {
            (0, can_reuse) => Self::Ephemeral(endpoint.addr, can_reuse),
            (_, true) => Self::CanReuse(endpoint.addr, port),
            (_, false) => Self::Specified(endpoint.addr, port),
        }
    }

    pub(super) fn can_reuse(&self) -> bool {
        matches!(self, Self::CanReuse(..)) || matches!(self, Self::Ephemeral(_, true))
    }

    pub(super) fn port(&self) -> Option<u16> {
        match self {
            Self::CanReuse(_, port) | Self::Specified(_, port) | Self::Backlog(_, port) => Some(*port),
            Self::Ephemeral(_, _) => None,
        }
    }

    pub(super) fn addr(&self) -> Option<IpAddress> {
        match self {
            Self::CanReuse(addr, _) | Self::Specified(addr, _) | Self::Ephemeral(addr, _) | Self::Backlog(addr, _) => Some(*addr),
        }
    }
}
