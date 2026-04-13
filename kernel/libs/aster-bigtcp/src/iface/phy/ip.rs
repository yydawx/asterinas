// SPDX-License-Identifier: MPL-2.0

use alloc::{string::String, sync::Arc};

use smoltcp::{
    iface::Config,
    phy::{Device, TxToken},
    wire::{self, Ipv4Cidr, Ipv6Cidr, Ipv4Packet},
};

use crate::{
    device::WithDevice,
    ext::Ext,
    iface::{
        Iface, ScheduleNextPoll,
        common::{IfaceCommon, InterfaceFlags, InterfaceType},
        iface::internal::IfaceInternal,
        time::get_network_timestamp,
    },
};

pub struct IpIface<D, E: Ext> {
    driver: D,
    common: IfaceCommon<E>,
}

impl<D: WithDevice, E: Ext> IpIface<D, E> {
    pub fn new(
        driver: D,
        ip_cidr: Ipv4Cidr,
        name: String,
        sched_poll: E::ScheduleNextPoll,
        type_: InterfaceType,
        flags: InterfaceFlags,
    ) -> Arc<Self> {
        Self::new_with_ipv6(
            driver,
            ip_cidr,
            None,
            name,
            sched_poll,
            type_,
            flags,
        )
    }

    pub fn new_with_ipv6(
        driver: D,
        ip_cidr: Ipv4Cidr,
        ipv6_cidr: Option<Ipv6Cidr>,
        name: String,
        sched_poll: E::ScheduleNextPoll,
        type_: InterfaceType,
        flags: InterfaceFlags,
    ) -> Arc<Self> {
        let interface = driver.with(|device| {
            let config = Config::new(smoltcp::wire::HardwareAddress::Ip);
            let now = get_network_timestamp();

            let mut interface = smoltcp::iface::Interface::new(config, device, now);
            interface.update_ip_addrs(|ip_addrs| {
                debug_assert!(ip_addrs.is_empty());
                ip_addrs.push(wire::IpCidr::Ipv4(ip_cidr)).unwrap();
                if let Some(ipv6_cidr) = ipv6_cidr {
                    ip_addrs.push(wire::IpCidr::Ipv6(ipv6_cidr)).unwrap();
                }
            });
            interface
        });

        let common = IfaceCommon::new(name, type_, flags, interface, sched_poll);

        Arc::new(Self { driver, common })
    }
}

impl<D, E: Ext> IfaceInternal<E> for IpIface<D, E> {
    fn common(&self) -> &IfaceCommon<E> {
        &self.common
    }
}

impl<D: WithDevice + 'static, E: Ext> Iface<E> for IpIface<D, E> {
    fn poll(&self) {
        self.driver.with(|device| {
            let next_poll = self.common.poll(
                device,
                |data, _iface_cx, tx_token| {
                    // 尝试解析为IPv4数据包
                    if let Ok(ipv4_pkt) = Ipv4Packet::new_checked(data) {
                        return Some((ipv4_pkt, tx_token));
                    }
                    // 尝试解析为IPv6数据包
                    if let Ok(_ipv6_pkt) = smoltcp::wire::Ipv6Packet::new_checked(data) {
                        // 暂时返回None，因为poll_ingress只处理IPv4
                        // 后续需要修改poll_ingress以支持IPv6
                        None
                    } else {
                        None
                    }
                },
                |pkt, iface_cx, tx_token| {
                    let ip_repr = pkt.ip_repr();
                    tx_token.consume(ip_repr.buffer_len(), |buffer| {
                        ip_repr.emit(&mut buffer[..], &iface_cx.checksum_caps());
                        pkt.emit_payload(
                            &ip_repr,
                            &mut buffer[ip_repr.header_len()..],
                            &iface_cx.caps,
                        );
                    });
                },
            );
            self.common.sched_poll().schedule_next_poll(next_poll);
        });
    }

    fn mtu(&self) -> usize {
        self.driver
            .with(|device| device.capabilities().max_transmission_unit)
    }
}
