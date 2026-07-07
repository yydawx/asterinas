// SPDX-License-Identifier: MPL-2.0

pub mod iface;
pub mod socket;
pub mod uts_ns;

pub fn init() {
    iface::init();
    socket::netlink::init();
    socket::vsock::init();
}

/// Lazy init should be called after spawning init thread.
pub fn init_in_first_kthread() {
    iface::init_in_first_kthread();
    // Virtio devices are probed during component init which runs right
    // before this function.  Try to pick up the Virtio-Net device now.
    iface::try_init_virtio_iface();
}
