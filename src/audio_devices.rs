//! Playback devices (Windows audio endpoints) for the volume pop-up's source:
//! `pro_volume_device` empty = the Windows default device (followed when it
//! changes), otherwise a device name (or part of it) as listed by `list()`.

/// One active playback device.
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    /// Endpoint ID (stable across reboots).
    pub id: String,
    /// Name as shown in Windows sound settings, e.g. "Speakers (Realtek(R) Audio)".
    pub name: String,
}

#[cfg(windows)]
mod imp {
    use super::Device;
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ};

    fn enumerator() -> Option<IMMDeviceEnumerator> {
        unsafe {
            // Fails harmlessly when COM is already initialized differently on this thread.
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()
        }
    }

    fn id_of(d: &IMMDevice) -> Option<String> {
        unsafe {
            let p = d.GetId().ok()?;
            let s = p.to_string().ok();
            CoTaskMemFree(Some(p.0 as *const _));
            s
        }
    }

    fn name_of(d: &IMMDevice) -> Option<String> {
        unsafe {
            let store = d.OpenPropertyStore(STGM_READ).ok()?;
            let v = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
            let s = v.to_string();
            (!s.is_empty()).then_some(s)
        }
    }

    pub fn list() -> Vec<Device> {
        let mut out = Vec::new();
        let Some(e) = enumerator() else { return out };
        unsafe {
            let Ok(c) = e.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) else { return out };
            let n = c.GetCount().unwrap_or(0);
            for i in 0..n {
                if let Ok(d) = c.Item(i) {
                    if let Some(id) = id_of(&d) {
                        let name = name_of(&d).unwrap_or_else(|| id.clone());
                        out.push(Device { id, name });
                    }
                }
            }
        }
        out
    }

    pub fn default_id() -> Option<String> {
        let e = enumerator()?;
        let d = unsafe { e.GetDefaultAudioEndpoint(eRender, eConsole) }.ok()?;
        id_of(&d)
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Device;
    pub fn list() -> Vec<Device> {
        Vec::new()
    }
    pub fn default_id() -> Option<String> {
        None
    }
}

/// Every active playback device.
pub fn list() -> Vec<Device> {
    imp::list()
}

/// The endpoint to watch for `pro_volume_device`: the Windows default when `wanted`
/// is empty or "default", else the first device whose name equals (or, failing
/// that, contains) `wanted`, ignoring case. `None` if nothing matches.
pub fn resolve(wanted: &str) -> Option<String> {
    let w = wanted.trim().to_lowercase();
    if w.is_empty() || w == "default" {
        return imp::default_id();
    }
    let all = list();
    all.iter()
        .find(|d| d.name.to_lowercase() == w)
        .or_else(|| all.iter().find(|d| d.name.to_lowercase().contains(&w)))
        .map(|d| d.id.clone())
}
