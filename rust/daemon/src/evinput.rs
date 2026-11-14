//! evdev reading + uinput writing via raw ioctls (port of evinput.py).

use std::fs;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::path::Path;

pub const EV_SYN: u16 = 0;
pub const EV_KEY: u16 = 1;
pub const EV_REL: u16 = 2;
pub const EV_MSC: u16 = 4;
pub const SYN_REPORT: u16 = 0;
pub const REL_X: u16 = 0;
pub const REL_Y: u16 = 1;
pub const REL_HWHEEL: u16 = 6;
pub const REL_WHEEL: u16 = 8;
pub const REL_WHEEL_HI_RES: u16 = 11;
pub const REL_HWHEEL_HI_RES: u16 = 12;
pub const MSC_SCAN: u16 = 4;

const EVIOCGRAB: libc::c_ulong = 0x4004_4590;
const UI_SET_EVBIT: libc::c_ulong = 0x4004_5564;
const UI_SET_KEYBIT: libc::c_ulong = 0x4004_5565;
const UI_SET_RELBIT: libc::c_ulong = 0x4004_5566;
const UI_SET_MSCBIT: libc::c_ulong = 0x4004_5568;
const UI_DEV_SETUP: libc::c_ulong = 0x405c_5503; // _IOW('U', 3, struct uinput_setup /* 92 bytes */)
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
const UI_DEV_DESTROY: libc::c_ulong = 0x5502;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct InputEvent {
    pub sec: i64,
    pub usec: i64,
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}
pub const EVENT_SIZE: usize = std::mem::size_of::<InputEvent>();

#[repr(C)]
struct UinputSetup {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
    name: [u8; 80],
    ff_effects_max: u32,
}

fn read_sys(p: &Path) -> String {
    fs::read_to_string(p).map(|s| s.trim().to_string()).unwrap_or_default()
}

/// Parse a sysfs capability bitmask ("1f0000 0 0 0 0") into bit numbers.
fn bitmask(text: &str) -> Vec<u32> {
    let mut bits = vec![];
    for (i, word) in text.split_whitespace().rev().enumerate() {
        let mut v = u64::from_str_radix(word, 16).unwrap_or(0);
        let mut b = 0;
        while v != 0 {
            if v & 1 == 1 {
                bits.push(i as u32 * 64 + b);
            }
            v >>= 1;
            b += 1;
        }
    }
    bits
}

/// (/dev/input/eventN, name) of the physical mouse interface (BTN_LEFT + REL_X), skipping virtual devices.
pub fn find_mouse_event_node(product: &str) -> Option<(String, String)> {
    let mut entries: Vec<_> = fs::read_dir("/sys/class/input").ok()?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let fname = e.file_name().to_string_lossy().to_string();
        if !fname.starts_with("event") {
            continue;
        }
        let dev = e.path().join("device");
        if read_sys(&dev.join("id/vendor")).to_lowercase() != "1532"
            || read_sys(&dev.join("id/product")).to_lowercase() != product
            || !read_sys(&dev.join("phys")).starts_with("usb-")
        {
            continue;
        }
        let keys = bitmask(&read_sys(&dev.join("capabilities/key")));
        let rel = bitmask(&read_sys(&dev.join("capabilities/rel")));
        if keys.contains(&272) && rel.contains(&0) {
            return Some((format!("/dev/input/{fname}"), read_sys(&dev.join("name"))));
        }
    }
    None
}

/// BTN_* codes (272..279) the physical mouse reports; None when the mouse is not found.
pub fn mouse_buttons(product: &str) -> Option<Vec<u16>> {
    let (path, _) = find_mouse_event_node(product)?;
    let name = Path::new(&path).file_name()?.to_string_lossy().to_string();
    let caps = read_sys(&Path::new("/sys/class/input").join(name).join("device/capabilities/key"));
    Some(bitmask(&caps).into_iter().filter(|b| (272..=279).contains(b)).map(|b| b as u16).collect())
}

pub struct EventReader {
    fd: OwnedFd,
    grabbed: bool,
}

impl EventReader {
    pub fn open(path: &str) -> io::Result<Self> {
        let c = std::ffi::CString::new(path).unwrap();
        let fd = unsafe { libc::open(c.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(EventReader { fd: unsafe { OwnedFd::from_raw_fd(fd) }, grabbed: false })
    }

    pub fn raw(&self) -> RawFd {
        self.fd.as_raw_fd()
    }

    pub fn grab(&mut self) -> io::Result<()> {
        if unsafe { libc::ioctl(self.raw(), EVIOCGRAB, 1 as libc::c_int) } < 0 {
            return Err(io::Error::last_os_error());
        }
        self.grabbed = true;
        Ok(())
    }

    /// Read all available events into `buf`; Err on unplug (ENODEV).
    pub fn read(&self, buf: &mut [InputEvent; 64]) -> io::Result<usize> {
        let n = unsafe { libc::read(self.raw(), buf.as_mut_ptr() as *mut libc::c_void, EVENT_SIZE * 64) };
        if n < 0 {
            let e = io::Error::last_os_error();
            return if e.kind() == io::ErrorKind::WouldBlock { Ok(0) } else { Err(e) };
        }
        Ok(n as usize / EVENT_SIZE)
    }
}

impl Drop for EventReader {
    fn drop(&mut self) {
        if self.grabbed {
            unsafe { libc::ioctl(self.raw(), EVIOCGRAB, 0 as libc::c_int) };
        }
    }
}

pub struct UInput {
    fd: OwnedFd,
}

impl UInput {
    pub fn create(name: &str, keys: &[u16], rels: &[u16], msc: &[u16], vendor: u16, product: u16) -> io::Result<Self> {
        let c = std::ffi::CString::new("/dev/uinput").unwrap();
        let raw = unsafe { libc::open(c.as_ptr(), libc::O_WRONLY | libc::O_NONBLOCK | libc::O_CLOEXEC) };
        if raw < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = unsafe { OwnedFd::from_raw_fd(raw) };
        let ioc = |req: libc::c_ulong, v: libc::c_int| -> io::Result<()> {
            if unsafe { libc::ioctl(raw, req, v) } < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
        };
        if !keys.is_empty() {
            ioc(UI_SET_EVBIT, EV_KEY as i32)?;
            for k in keys { ioc(UI_SET_KEYBIT, *k as i32)?; }
        }
        if !rels.is_empty() {
            ioc(UI_SET_EVBIT, EV_REL as i32)?;
            for r in rels { ioc(UI_SET_RELBIT, *r as i32)?; }
        }
        if !msc.is_empty() {
            ioc(UI_SET_EVBIT, EV_MSC as i32)?;
            for m in msc { ioc(UI_SET_MSCBIT, *m as i32)?; }
        }
        let mut setup = UinputSetup { bustype: 3, vendor, product, version: 1, name: [0; 80], ff_effects_max: 0 };
        let nb = name.as_bytes();
        setup.name[..nb.len().min(79)].copy_from_slice(&nb[..nb.len().min(79)]);
        if unsafe { libc::ioctl(raw, UI_DEV_SETUP, &setup as *const UinputSetup) } < 0 {
            return Err(io::Error::last_os_error());
        }
        if unsafe { libc::ioctl(raw, UI_DEV_CREATE) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(UInput { fd })
    }

    pub fn write_events(&self, events: &[InputEvent]) -> io::Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        let n = unsafe {
            libc::write(self.fd.as_raw_fd(), events.as_ptr() as *const libc::c_void, events.len() * EVENT_SIZE)
        };
        if n < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }

    pub fn key(&self, code: u16, value: i32) -> io::Result<()> {
        self.write_events(&[ev(EV_KEY, code, value), ev(EV_SYN, SYN_REPORT, 0)])
    }
}

impl Drop for UInput {
    fn drop(&mut self) {
        unsafe { libc::ioctl(self.fd.as_raw_fd(), UI_DEV_DESTROY) };
    }
}

pub fn ev(kind: u16, code: u16, value: i32) -> InputEvent {
    InputEvent { sec: 0, usec: 0, kind, code, value }
}

pub fn virtual_mouse(name: &str, product: u16) -> io::Result<UInput> {
    let keys: Vec<u16> = (272..280).collect();
    UInput::create(
        name,
        &keys,
        &[REL_X, REL_Y, REL_WHEEL, REL_HWHEEL, REL_WHEEL_HI_RES, REL_HWHEEL_HI_RES],
        &[MSC_SCAN],
        0x1532,
        product,
    )
}

pub fn virtual_keyboard(name: &str) -> io::Result<UInput> {
    let keys: Vec<u16> = (1..249).collect();
    UInput::create(name, &keys, &[], &[], 0x1532, 0xB2FF)
}
