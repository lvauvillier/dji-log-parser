use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::layout::details::Details;
use crate::record::component_serial::{ComponentSerial, ComponentType};
use crate::record::recover::Recover;

/// Serial numbers of the components, from the serial number and aircraft information records, or
/// else from the recover records and the log details.
///
/// The serial number of the aircraft of the details and of the recover records is cut to 16
/// characters, recent aircraft have 20.
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameSerial {
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub aircraft: Option<String>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub remote_controller: Option<String>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub battery1: Option<String>,
    /// On aircraft with several batteries
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub battery2: Option<String>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub camera1: Option<String>,
    /// On aircraft with several cameras
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub camera2: Option<String>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub gimbal1: Option<String>,
    /// On aircraft with several gimbals
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub gimbal2: Option<String>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub ssd: Option<String>,
}

impl FrameSerial {
    /// Serial numbers of the log details
    pub(crate) fn from_details(details: &Details) -> Self {
        FrameSerial {
            aircraft: non_empty(&details.aircraft_sn),
            remote_controller: non_empty(&details.rc_sn),
            battery1: non_empty(&details.battery_sn),
            camera1: non_empty(&details.camera_sn),
            ..FrameSerial::default()
        }
    }

    /// Replaces the serial numbers of a recover record, which are those of the log details, but
    /// also set on old logs whose details only have zeros.
    pub(crate) fn update_from_recover(&mut self, recover: &Recover) {
        for (field, serial) in [
            (&mut self.aircraft, &recover.aircraft_sn),
            (&mut self.remote_controller, &recover.rc_sn),
            (&mut self.battery1, &recover.battery_sn),
            (&mut self.camera1, &recover.camera_sn),
        ] {
            set(field, serial);
        }
    }

    /// Sets the serial number of a component, the components after the second of their type are
    /// ignored.
    pub(crate) fn update(&mut self, component: &ComponentSerial) {
        let first = component.index == 0;
        let second = component.index == 1;
        let field = match component.component_type {
            ComponentType::Aircraft => &mut self.aircraft,
            ComponentType::RC => &mut self.remote_controller,
            ComponentType::Ssd => &mut self.ssd,
            ComponentType::Battery if first => &mut self.battery1,
            ComponentType::Battery if second => &mut self.battery2,
            ComponentType::Camera if first => &mut self.camera1,
            ComponentType::Camera if second => &mut self.camera2,
            ComponentType::Gimbal if first => &mut self.gimbal1,
            ComponentType::Gimbal if second => &mut self.gimbal2,
            _ => return,
        };
        set(field, &component.serial);
    }

    /// Sets the serial numbers of the batteries of the aircraft information records, written by
    /// the aircraft with several batteries without the serial number records
    pub(crate) fn update_batteries(&mut self, batteries: &[String]) {
        for (field, serial) in [&mut self.battery1, &mut self.battery2]
            .into_iter()
            .zip(batteries)
        {
            set(field, serial);
        }
    }

    /// Serial numbers of `self`, completed by those of `other`
    pub(crate) fn or(&self, other: &FrameSerial) -> FrameSerial {
        let or = |a: &Option<String>, b: &Option<String>| a.clone().or_else(|| b.clone());
        FrameSerial {
            aircraft: or(&self.aircraft, &other.aircraft),
            remote_controller: or(&self.remote_controller, &other.remote_controller),
            battery1: or(&self.battery1, &other.battery1),
            battery2: or(&self.battery2, &other.battery2),
            camera1: or(&self.camera1, &other.camera1),
            camera2: or(&self.camera2, &other.camera2),
            gimbal1: or(&self.gimbal1, &other.gimbal1),
            gimbal2: or(&self.gimbal2, &other.gimbal2),
            ssd: or(&self.ssd, &other.ssd),
        }
    }
}

/// Sets a serial number, unless it is a serial number cut short of the current one, as some
/// records repeat
fn set(field: &mut Option<String>, serial: &str) {
    let Some(serial) = non_empty(serial) else {
        return;
    };
    if !field
        .as_ref()
        .is_some_and(|current| current.starts_with(&serial))
    {
        *field = Some(serial);
    }
}

/// A serial number, none when it is empty, only zeros, or not printable as on some old logs
fn non_empty(serial: &str) -> Option<String> {
    let serial = serial.trim();
    let is_valid = serial.chars().any(|c| c != '0') && serial.bytes().all(|b| b.is_ascii_graphic());
    is_valid.then(|| serial.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(component_type: ComponentType, index: u8, serial: &str) -> ComponentSerial {
        ComponentSerial {
            component_type,
            index,
            serial: serial.to_owned(),
        }
    }

    #[test]
    fn batteries_by_index() {
        let mut serial = FrameSerial::default();
        serial.update(&component(ComponentType::Battery, 0, "09UADBH03101G8"));
        serial.update(&component(ComponentType::Battery, 1, "09UADBH03101JU"));
        serial.update(&component(ComponentType::Battery, 2, "09UADBH03101XX"));

        assert_eq!(serial.battery1.as_deref(), Some("09UADBH03101G8"));
        assert_eq!(serial.battery2.as_deref(), Some("09UADBH03101JU"));
    }

    #[test]
    fn batteries_of_the_aircraft_information() {
        let mut serial = FrameSerial::default();
        serial.update_batteries(&["4BUPKCMDA22072".to_owned(), "4BUPKCMDA22062".to_owned()]);

        assert_eq!(serial.battery1.as_deref(), Some("4BUPKCMDA22072"));
        assert_eq!(serial.battery2.as_deref(), Some("4BUPKCMDA22062"));
    }

    #[test]
    fn serial_cut_short_is_ignored() {
        let mut serial = FrameSerial::default();
        serial.update(&component(ComponentType::Camera, 0, "0BDDF5H00100RA"));
        serial.update(&component(ComponentType::Camera, 0, "0BDDF5H00"));

        assert_eq!(serial.camera1.as_deref(), Some("0BDDF5H00100RA"));
    }

    #[test]
    fn records_come_before_the_details() {
        let records = FrameSerial {
            aircraft: Some("1581F6GKB235K00403NN".to_owned()),
            ..FrameSerial::default()
        };
        let details = FrameSerial {
            aircraft: Some("1581F6GKB235K004".to_owned()),
            remote_controller: Some("4LGZL4M0070414".to_owned()),
            ..FrameSerial::default()
        };

        let serial = records.or(&details);
        assert_eq!(serial.aircraft.as_deref(), Some("1581F6GKB235K00403NN"));
        assert_eq!(serial.remote_controller.as_deref(), Some("4LGZL4M0070414"));
    }

    #[test]
    fn invalid_serials_are_none() {
        assert_eq!(non_empty(""), None);
        assert_eq!(non_empty("0000000000"), None);
        assert_eq!(non_empty("\u{5}"), None);
        assert_eq!(non_empty(" 03Z04313 "), Some("03Z04313".to_owned()));
    }
}
