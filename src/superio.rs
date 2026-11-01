//! Motherboard fan speeds read directly from a **Nuvoton NCT679x** Super I/O chip
//! (e.g. the NCT6799D on ASUS TUF GAMING B650-PLUS), through PawnIO and its official,
//! signed `LpcIO` module — no HWiNFO or LibreHardwareMonitor needed.
//!
//! Register layout follows LibreHardwareMonitor's `Nct677X.cs` / Linux `nct6775`:
//! - config ports 0x2E/0x2F, enter with 0x87 0x87, leave with 0xAA;
//! - chip ID at config register 0x20, hardware monitor = logical device 0x0B,
//!   its I/O base at 0x60/0x61, the "I/O space lock" bit 4 of register 0x28;
//! - hardware monitor registers are banked: address port = base + 5, data port =
//!   base + 6, bank select register 0x4E;
//! - fan speeds (RPM, 16-bit big endian) at 0x4C0, 0x4C2, ... 0x4CA and 0x4CE.
//!
//! On ASUS boards "Fan #2" is CPU_FAN (LibreHardwareMonitor's naming). Requires
//! Administrator rights (like the CPU temperature) and the PawnIO driver.

#[cfg(windows)]
mod imp {
    use crate::pawnio::PawnIo;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

    /// Official PawnIO module for LPC Super I/O access (PawnIO.Modules, LGPL-2.1-or-later),
    /// downloaded by the build workflow.
    static LPCIO_MODULE: &[u8] = include_bytes!("resources/LpcIO.bin");

    /// Shared with HWiNFO, LibreHardwareMonitor, FanControl, ... so two programs never
    /// interleave Super I/O accesses.
    const ISA_MUTEX: &str = "Global\\Access_ISABUS.HTP.Method";

    const FAN_REGS: [u16; 7] = [0x4C0, 0x4C2, 0x4C4, 0x4C6, 0x4C8, 0x4CA, 0x4CE];

    struct Guard(HANDLE);
    impl Guard {
        fn acquire() -> Option<Guard> {
            let wide: Vec<u16> = ISA_MUTEX.encode_utf16().chain(std::iter::once(0)).collect();
            let h = unsafe { CreateMutexW(None, false, PCWSTR(wide.as_ptr())) }.ok()?;
            // 0 = signaled, 0x80 = abandoned (both mean we own it); wait at most 200 ms.
            let w = unsafe { WaitForSingleObject(h, 200) }.0;
            if w == 0 || w == 0x80 {
                Some(Guard(h))
            } else {
                unsafe {
                    let _ = CloseHandle(h);
                }
                None
            }
        }
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe {
                let _ = ReleaseMutex(self.0);
                let _ = CloseHandle(self.0);
            }
        }
    }

    pub struct SuperIo {
        io: PawnIo,
        base: u16,
        pub chip: String,
    }

    impl SuperIo {
        fn sio_in(&self, reg: u8) -> Option<u8> {
            self.io.execute("ioctl_superio_inb", &[reg as u64], 1)?.first().map(|v| *v as u8)
        }
        fn sio_out(&self, reg: u8, val: u8) -> Option<()> {
            self.io.execute("ioctl_superio_outb", &[reg as u64, val as u64], 0).map(|_| ())
        }
        fn port_out(&self, port: u16, val: u8) -> Option<()> {
            self.io.execute("ioctl_pio_outb", &[port as u64, val as u64], 0).map(|_| ())
        }
        fn port_in(&self, port: u16) -> Option<u8> {
            self.io.execute("ioctl_pio_inb", &[port as u64], 1)?.first().map(|v| *v as u8)
        }

        /// Finds a Nuvoton NCT679x on config port 0x2E (slot 0), else 0x4E (slot 1).
        pub fn open() -> Option<SuperIo> {
            let mut s = SuperIo { io: PawnIo::open_with_module(LPCIO_MODULE)?, base: 0, chip: String::new() };
            let _g = Guard::acquire()?;
            for (slot, port) in [(0u64, 0x2Eu16), (1, 0x4E)] {
                if s.try_slot(slot, port) {
                    return Some(s);
                }
            }
            None
        }

        fn try_slot(&mut self, slot: u64, port: u16) -> bool {
            if self.io.execute("ioctl_select_slot", &[slot], 0).is_none() {
                return false;
            }
            // Enter the Nuvoton configuration mode.
            let _ = self.port_out(port, 0x87);
            let _ = self.port_out(port, 0x87);
            let id = self.sio_in(0x20).unwrap_or(0);
            let rev = self.sio_in(0x21).unwrap_or(0);
            let mut base = 0u16;
            if (0xC8..=0xD8).contains(&id) && self.io.execute("ioctl_find_bars", &[], 0).is_some() {
                // Hardware monitor logical device and its I/O base.
                let _ = self.sio_out(0x07, 0x0B);
                base = ((self.sio_in(0x60).unwrap_or(0) as u16) << 8) | self.sio_in(0x61).unwrap_or(0) as u16;
                // Clear the I/O space lock, as LibreHardwareMonitor does for NCT679x.
                if let Some(v) = self.sio_in(0x28) {
                    if v & 0x10 != 0 {
                        let _ = self.sio_out(0x28, v & !0x10);
                    }
                }
            }
            // Leave the configuration mode.
            let _ = self.port_out(port, 0xAA);
            if base < 0x100 || base == 0xFFFF {
                return false;
            }
            self.base = base & 0xFFF8;
            self.chip = format!("Nuvoton chip {id:02X}{rev:02X} at {port:#X}, monitor base {:#X}", self.base);
            true
        }

        fn read(&self, reg: u16) -> Option<u8> {
            self.port_out(self.base + 5, 0x4E)?;
            self.port_out(self.base + 6, (reg >> 8) as u8)?;
            self.port_out(self.base + 5, (reg & 0xFF) as u8)?;
            self.port_in(self.base + 6)
        }

        /// RPM of fan headers #1..#7 (`None` = could not read).
        pub fn fans(&self) -> [Option<u32>; 7] {
            let mut out = [None; 7];
            let Some(_g) = Guard::acquire() else { return out };
            for (i, r) in FAN_REGS.iter().enumerate() {
                if let (Some(hi), Some(lo)) = (self.read(*r), self.read(*r + 1)) {
                    let v = ((hi as u32) << 8) | lo as u32;
                    out[i] = Some(if v == 0xFFFF { 0 } else { v });
                }
            }
            out
        }
    }

    // SAFETY: used from the main loop only.
    unsafe impl Send for SuperIo {}
}

#[cfg(not(windows))]
mod imp {
    pub struct SuperIo {
        pub chip: String,
    }
    impl SuperIo {
        pub fn open() -> Option<SuperIo> {
            None
        }
        pub fn fans(&self) -> [Option<u32>; 7] {
            [None; 7]
        }
    }
}

pub use imp::SuperIo;

/// Which header `pro_fan_sensor` means: "Fan #3" / "fan3" / "3" → index 2; empty,
/// "CPU" or "CPU Fan" → Fan #2 (CPU_FAN on ASUS boards).
pub fn fan_index(pick: &str) -> usize {
    let p = pick.trim().to_ascii_lowercase();
    let digits: String = p.chars().filter(|c| c.is_ascii_digit()).collect();
    if p.starts_with("fan") || (!digits.is_empty() && digits.len() == p.len()) {
        if let Ok(n) = digits.parse::<usize>() {
            if (1..=7).contains(&n) {
                return n - 1;
            }
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::fan_index;
    #[test]
    fn picks() {
        assert_eq!(fan_index(""), 1);
        assert_eq!(fan_index("CPU"), 1);
        assert_eq!(fan_index("CPU Fan"), 1);
        assert_eq!(fan_index("Fan #3"), 2);
        assert_eq!(fan_index("fan7"), 6);
        assert_eq!(fan_index("1"), 0);
    }
}
