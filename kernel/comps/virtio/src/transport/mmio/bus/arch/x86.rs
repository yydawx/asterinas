// SPDX-License-Identifier: MPL-2.0

pub(super) use ostd::arch::irq::MappedIrqLine;
use ostd::{arch::irq::IRQ_CHIP, debug};

use crate::transport::mmio::bus::MmioRegisterError;

/// Known virtio MMIO device addresses and their IOAPIC pin (GSI) numbers.
/// These must match the virtio_cfg.json configuration used by the hypervisor's
/// virtio daemon.
const KNOWN_DEVICES: &[(usize, u32)] = &[
    (0xFEB0_0000, 16), // console
    (0xFEB0_0200, 17), // net
];

pub(super) fn probe_for_device() {
    let irq_chip = match IRQ_CHIP.get() {
        Some(chip) => chip,
        None => {
            debug!("Skip MMIO detection because there is no IRQ chip");
            return;
        }
    };

    if irq_chip.count_io_apics() == 0 {
        debug!("Skip MMIO detection because there are no I/O APICs");
        return;
    }

    for &(mmio_base, gsi) in KNOWN_DEVICES {
        ostd::info!("Probing virtio MMIO device at {:#x} (GSI {})", mmio_base, gsi);
        match super::try_register_mmio_device(mmio_base..(mmio_base + 0x200), |irq_line| {
            irq_chip.map_gsi_pin_to(irq_line, gsi)
        }) {
            Ok(()) => ostd::info!("Registered virtio MMIO device at {:#x} (GSI {})", mmio_base, gsi),
            Err(e) => ostd::info!(
                "No virtio MMIO device at {:#x}: {:?}",
                mmio_base, e
            ),
        }
    }
}

impl MmioRegisterError {
    /// Returns `true` if it should terminate a linear MMIO scan.
    fn is_fatal(self) -> bool {
        matches!(self, Self::MmioUnavailable | Self::MagicMismatch)
    }
}
