// SPDX-License-Identifier: MPL-2.0

use aster_bigtcp::{
    errors::BindError,
    iface::BindPortConfig,
    wire::{IpAddress, IpEndpoint, Ipv6Address},
};

use crate::{
    net::iface::{BoundPort, Iface, iter_all_ifaces, loopback_iface, virtio_iface},
    prelude::*,
};

pub(super) fn get_iface_to_bind(ip_addr: &IpAddress) -> Option<Arc<Iface>> {
    match *ip_addr {
        IpAddress::Ipv4(ipv4_addr) => {
            iter_all_ifaces()
                .find(|iface| {
                    if let Some(iface_ipv4_addr) = iface.ipv4_addr() {
                        iface_ipv4_addr == ipv4_addr
                    } else {
                        false
                    }
                })
                .map(Clone::clone)
        }
        IpAddress::Ipv6(ipv6_addr) => {
            iter_all_ifaces()
                .find(|iface| {
                    if let Some(iface_ipv6_addr) = iface.ipv6_addr() {
                        iface_ipv6_addr == ipv6_addr
                    } else {
                        false
                    }
                })
                .map(Clone::clone)
        }
    }
}

/// Get a suitable iface to deal with sendto/connect request if the socket is not bound to an iface.
/// If the remote address is the same as that of some iface, we will use the iface.
/// Otherwise, we will use a default interface.
fn get_ephemeral_iface(remote_ip_addr: &IpAddress) -> Arc<Iface> {
    match *remote_ip_addr {
        IpAddress::Ipv4(remote_ipv4_addr) => {
            if let Some(iface) = iter_all_ifaces().find(|iface| {
                if let Some(iface_ipv4_addr) = iface.ipv4_addr() {
                    iface_ipv4_addr == remote_ipv4_addr
                } else {
                    false
                }
            }) {
                emerg!("IPv4: found matching iface {} for remote {}", iface.name(), remote_ipv4_addr);
                return iface.clone();
            }

            // FIXME: Instead of hardcoding the rules here, we should choose the
            // default interface according to the routing table.
            if let Some(virtio_iface) = virtio_iface() {
                emerg!("IPv4: using virtio_iface (eth0)");
                virtio_iface.clone()
            } else {
                emerg!("IPv4: using loopback_iface");
                loopback_iface().clone()
            }
        }
        IpAddress::Ipv6(remote_ipv6_addr) => {
            if let Some(iface) = iter_all_ifaces().find(|iface| {
                if let Some(iface_ipv6_addr) = iface.ipv6_addr() {
                    iface_ipv6_addr == remote_ipv6_addr
                } else {
                    false
                }
            }) {
                emerg!("IPv6: found matching iface {} for remote {}", iface.name(), remote_ipv6_addr);
                return iface.clone();
            }

            // For IPv6,virtio_iface might not have IPv6 address configured
            if let Some(virtio_iface) = virtio_iface() {
                emerg!("IPv6: virtio_iface {:?} has ipv6: {:?}", virtio_iface.name(), virtio_iface.ipv6_addr());
                // Even if virtio has no IPv6, we can't use loopback for external connections
                if virtio_iface.ipv6_addr().is_some() {
                    return virtio_iface.clone();
                }
            }

            emerg!("IPv6: falling back to loopback_iface (lo)");
            loopback_iface().clone()
        }
    }
}

pub(super) fn bind_port(endpoint: &IpEndpoint, can_reuse: bool) -> Result<BoundPort> {
    let iface = match get_iface_to_bind(&endpoint.addr) {
        Some(iface) => {
            emerg!("bind_port: found iface {} for endpoint {:?}", iface.name(), endpoint);
            iface
        }
        None => {
            emerg!("bind_port: no iface found for endpoint {:?}", endpoint);
            return_errno_with_message!(
                Errno::EADDRNOTAVAIL,
                "the address is not available from the local machine"
            );
        }
    };

    let bind_port_config = BindPortConfig::new(*endpoint, can_reuse);

    emerg!("bind_port: calling iface.bind() on {}", iface.name());
    let bound_port = iface.bind(bind_port_config)?;
    emerg!("bind_port: iface.bind() returned port {}", bound_port.port());
    Ok(bound_port)
}

impl From<BindError> for Error {
    fn from(value: BindError) -> Self {
        match value {
            BindError::Exhausted => {
                Error::with_message(Errno::EAGAIN, "no ephemeral port is available")
            }
            BindError::InUse => {
                Error::with_message(Errno::EADDRINUSE, "the address is already in use")
            }
        }
    }
}

pub(super) fn get_ephemeral_endpoint(remote_endpoint: &IpEndpoint) -> IpEndpoint {
    match remote_endpoint.addr {
        IpAddress::Ipv4(_) => {
            let iface = get_ephemeral_iface(&remote_endpoint.addr);
            emerg!("IPv4 connect: using iface {} for remote {:?}", iface.name(), remote_endpoint.addr);
            let ip_addr = iface.ipv4_addr().unwrap();
            IpEndpoint::new(IpAddress::Ipv4(ip_addr), 0)
        }
        IpAddress::Ipv6(_) => {
            let iface = get_ephemeral_iface(&remote_endpoint.addr);
            emerg!("IPv6 connect: using iface {} (ipv6 addr: {:?}) for remote {:?}", iface.name(), iface.ipv6_addr(), remote_endpoint.addr);
            if let Some(ipv6_addr) = iface.ipv6_addr() {
                IpEndpoint::new(IpAddress::Ipv6(ipv6_addr), 0)
            } else {
                IpEndpoint::new(IpAddress::Ipv6(Ipv6Address::new(0, 0, 0, 0, 0, 0, 0, 0)), 0)
            }
        }
    }
}
