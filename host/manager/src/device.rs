//! Discovery and tracking of connected devices.
//!
//! Our nanos are CH340 usb-serial clones that do not report a unique serial
//! number, so `/dev/serial/by-id` symlinks are identical for all of them and
//! the plain `/dev/ttyUSBx` nodes are assigned by probe order (unstable
//! across replug and reboot).
//!
//! Instead devices are identified by the **physical usb port** they are
//! plugged into, via udev's stable `/dev/serial/by-path` symlinks. Moving a
//! nano to a different hub port changes its identity; as long as the wiring
//! stays put, identity survives reboots and re-enumeration.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Directory of stable per-port symlinks maintained by udev.
const BY_PATH: &str = "/dev/serial/by-path";

/// CH340 usb ids (arduino nano clones).
const CH340_VENDOR: &str = "1a86";
const CH340_PRODUCT: &str = "7523";

/// Stable identity of a connected device: the physical usb port it sits on
///
/// Stable until a device is physically moved
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(String);

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A connected arduino.
#[derive(Debug)]
pub struct Device {
    /// Stable id based on usb topology
    id: DeviceId,
    /// Currently resolved device node (`/dev/ttyUSBx`).
    ///
    /// Unstable across re-plug and reboot, unlike [`Device::id`].
    tty: PathBuf,
}

impl Device {
    /// All connected nanos, ordered by [`DeviceId`].
    pub fn connected() -> io::Result<Vec<Self>> {
        let entries = match fs::read_dir(BY_PATH) {
            Ok(entries) => entries,
            // no usb-serial devices connected at all
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };

        let mut devices = Vec::new();
        for entry in entries {
            let link = entry?.path();
            let Some(file_name) = link.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            // udev creates two symlinks per port, one encoding the usb
            // revision (`-usbv2-`); keep only the classic (revision-less) one
            if file_name.contains("usbv") {
                continue;
            }
            // names end in `-port<n>`; we only claim single-interface,
            // single-port converters, which is what a nano looks like
            let Some(id) = file_name.strip_suffix("-port0") else {
                continue;
            };
            // resolve first: the device may vanish mid-enumeration
            let Ok(tty) = link.canonicalize() else {
                continue;
            };
            if !is_ch340(&tty) {
                continue;
            }
            devices.push(Self {
                id: DeviceId(id.to_owned()),
                tty,
            });
        }
        devices.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        Ok(devices)
    }

    /// Stable device id
    ///
    /// Changes only when connection topology changes
    pub fn id(&self) -> &DeviceId {
        &self.id
    }

    /// The current device node
    pub fn tty(&self) -> &Path {
        &self.tty
    }
}

/// Whether the tty behind a `by-path` symlink is a CH340 usb-serial converter.
fn is_ch340(tty: &Path) -> bool {
    let Some(tty_name) = tty.file_name() else {
        return false;
    };
    // For usb-serial drivers (ch341, ftdi, ...) the sysfs chain is
    //   tty class device -> usb-serial port -> usb interface -> usb device
    // so `<tty>/device` is the port and the ids live on its grandparent,
    // the usb device (`<tty>/device/../..`).
    let usb_dev = Path::new("/sys/class/tty").join(tty_name).join("device");
    usb_attribute(&usb_dev, "idVendor").as_deref() == Some(CH340_VENDOR)
        && usb_attribute(&usb_dev, "idProduct").as_deref() == Some(CH340_PRODUCT)
}

/// Reads a usb device attribute (e.g. `idVendor`) from sysfs.
fn usb_attribute(usb_dev: &Path, attribute: &str) -> Option<String> {
    let path = usb_dev.join("../..").join(attribute);
    match fs::read_to_string(path) {
        Ok(value) => Some(value.trim().to_owned()),
        // attribute missing or device gone
        Err(_) => None,
    }
}
