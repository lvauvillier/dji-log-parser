use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::firmware::{Firmware, SenderType};

/// Firmware versions of the components, from version 8.
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameFirmware {
    /// Version of the flight controller
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
}

impl FrameFirmware {
    /// Sets the versions of a firmware record, the components after the second of their type are
    /// ignored, and so are the components of unknown types.
    pub(crate) fn update(&mut self, firmware: &Firmware) {
        for entry in &firmware.versions {
            let first = entry.index == 0;
            let second = entry.index == 1;
            let field = match entry.sender_type {
                SenderType::MC => &mut self.aircraft,
                SenderType::RC => &mut self.remote_controller,
                SenderType::Battery if first => &mut self.battery1,
                SenderType::Battery if second => &mut self.battery2,
                SenderType::Camera if first => &mut self.camera1,
                SenderType::Camera if second => &mut self.camera2,
                SenderType::Gimbal if first => &mut self.gimbal1,
                SenderType::Gimbal if second => &mut self.gimbal2,
                _ => continue,
            };
            *field = Some(entry.version.clone());
        }
    }
}
