use binrw::io::{Read, Seek};
use binrw::{BinRead, BinResult, Endian};
use num_enum::FromPrimitive;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

/// Vision system state: a list of entries, each identified by a key.
#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct VisionGroup {
    pub entries: Vec<VisionGroupEntry>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum VisionGroupEntry {
    ObstacleDetection(ObstacleDetection),
    ObstacleAvoidance(ObstacleAvoidance),
    Unknown { key: u32, data: Vec<u8> },
}

/// Obstacles detected by the sensors of one direction, in one or more sectors.
#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct ObstacleDetection {
    pub direction: SensorDirection,
    pub sectors: Vec<ObstacleSector>,
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum SensorDirection {
    Front = 0,
    Back = 1,
    Right = 2,
    Left = 3,
    Top = 4,
    Bottom = 5,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct ObstacleSector {
    /// meters, out of the sensor range (often 100) when nothing is detected
    pub distance: f32,
    pub warning_level: u8,
}

impl ObstacleDetection {
    /// Keys of the directions
    const KEYS: std::ops::RangeInclusive<u32> = 0..=5;

    /// Parses the sector count and direction, then the distance (`u16`, cm) and warning level of
    /// each sector. Recent apps add a counter after the sectors.
    fn parse(data: &[u8]) -> Option<Self> {
        let (header, sectors) = data.split_first()?;
        let count = usize::from(header & 0x1F);
        let (sectors, _) = sectors.get(..count * 3)?.as_chunks::<3>();
        Some(ObstacleDetection {
            direction: SensorDirection::from(header >> 5),
            sectors: sectors
                .iter()
                .map(|&[low, high, warning_level]| ObstacleSector {
                    distance: f32::from(u16::from_le_bytes([low, high])) / 100.0,
                    warning_level,
                })
                .collect(),
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct ObstacleAvoidance {
    pub is_obstacle_avoidance_enabled: bool,
    pub is_collision_avoidance_enabled: bool,
    pub is_avoiding_obstacle_legacy: bool,
    pub is_braking: bool,
    pub is_rth_obstacle_avoidance_enabled: bool,
    pub is_landing_confirmation_needed: bool,
    pub is_at_distance_limit: bool,
    pub is_at_airport_boundary: bool,
    pub is_avoiding_obstacle: bool,
    pub is_near_horizontal_limit: bool,
    pub is_avoiding_active_obstacle: bool,
    pub is_at_low_altitude_limit: bool,
    pub is_at_airport_altitude_limit: bool,
    pub is_ascent_limited_by_obstacle: bool,
    pub is_at_ground_limit: bool,
}

impl ObstacleAvoidance {
    const KEY: u32 = 8;

    fn from_flags(flags: u16) -> Self {
        let is_set = |bit: u16| flags & (1 << bit) != 0;
        ObstacleAvoidance {
            is_obstacle_avoidance_enabled: is_set(0),
            is_collision_avoidance_enabled: is_set(1),
            is_avoiding_obstacle_legacy: is_set(2),
            is_braking: is_set(3),
            is_rth_obstacle_avoidance_enabled: is_set(4),
            is_landing_confirmation_needed: is_set(5),
            is_at_distance_limit: is_set(6),
            is_at_airport_boundary: is_set(7),
            is_avoiding_obstacle: is_set(8),
            is_near_horizontal_limit: is_set(9),
            is_avoiding_active_obstacle: is_set(10),
            is_at_low_altitude_limit: is_set(11),
            is_at_airport_altitude_limit: is_set(12),
            is_ascent_limited_by_obstacle: is_set(13),
            is_at_ground_limit: is_set(14),
        }
    }
}

impl VisionGroupEntry {
    fn new(key: u32, data: &[u8]) -> Self {
        let entry = match (key, data) {
            (key, data) if ObstacleDetection::KEYS.contains(&key) => {
                ObstacleDetection::parse(data).map(VisionGroupEntry::ObstacleDetection)
            }
            (ObstacleAvoidance::KEY, [low, high, ..]) => Some(VisionGroupEntry::ObstacleAvoidance(
                ObstacleAvoidance::from_flags(u16::from_le_bytes([*low, *high])),
            )),
            _ => None,
        };
        entry.unwrap_or_else(|| VisionGroupEntry::Unknown {
            key,
            data: data.to_vec(),
        })
    }
}

/// Reads the group from its data length.
impl BinRead for VisionGroup {
    type Args<'a> = (u16,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        (length,): Self::Args<'_>,
    ) -> BinResult<Self> {
        let position = reader.stream_position()?;
        let mut data = vec![0; length.into()];
        reader.read_exact(&mut data)?;

        // Keys are written on 1 byte by older apps, on 4 bytes by newer ones. Entries with keys of
        // 1 byte can also be split as keys of 4 bytes, not the other way around.
        [1, 4]
            .into_iter()
            .find_map(|key_size| entries(&data, key_size))
            .map(|entries| VisionGroup { entries })
            .ok_or_else(|| binrw::Error::AssertFail {
                pos: position,
                message: "invalid vision group entries".to_owned(),
            })
    }
}

/// Splits the data into entries: a key, a length, the entry data and an end byte. Fails if the
/// data doesn't follow this layout with keys of `key_size` bytes.
fn entries(mut data: &[u8], key_size: usize) -> Option<Vec<VisionGroupEntry>> {
    const ENTRY_END: u8 = 0xFE;

    let mut entries = Vec::new();
    // The data can be followed by zeros
    while data.iter().any(|byte| *byte != 0) {
        let (key, rest) = data.split_at_checked(key_size)?;
        let (length, rest) = rest.split_first()?;
        let (entry, rest) = rest.split_at_checked((*length).into())?;
        let (end, rest) = rest.split_first()?;
        if *end != ENTRY_END {
            return None;
        }

        let mut key_bytes = [0; 4];
        key_bytes[..key_size].copy_from_slice(key);
        entries.push(VisionGroupEntry::new(u32::from_le_bytes(key_bytes), entry));
        data = rest;
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;

    fn read(data: &[u8]) -> VisionGroup {
        let length = data.len() as u16;
        VisionGroup::read_le_args(&mut Cursor::new(data), (length,)).unwrap()
    }

    fn obstacle_avoidance(group: &VisionGroup) -> &ObstacleAvoidance {
        group
            .entries
            .iter()
            .find_map(|entry| match entry {
                VisionGroupEntry::ObstacleAvoidance(state) => Some(state),
                _ => None,
            })
            .expect("no obstacle avoidance entry")
    }

    #[test]
    fn entries_with_keys_of_1_byte() {
        let group = read(&[
            0x08, 0x02, 0x13, 0x00, 0xfe, 0x09, 0x07, 0xf6, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x8b,
            0xfe,
        ]);

        assert_eq!(group.entries.len(), 2);
        let state = obstacle_avoidance(&group);
        assert!(state.is_obstacle_avoidance_enabled);
        assert!(state.is_collision_avoidance_enabled);
        assert!(state.is_rth_obstacle_avoidance_enabled);
        assert!(!state.is_landing_confirmation_needed);
        assert!(matches!(
            group.entries[1],
            VisionGroupEntry::Unknown { key: 9, ref data } if data.len() == 7
        ));
    }

    #[test]
    fn entries_with_keys_of_4_bytes_followed_by_zeros() {
        let group = read(&[
            0x08, 0x00, 0x00, 0x00, 0x03, 0x33, 0x00, 0x00, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);

        assert_eq!(group.entries.len(), 1);
        assert!(obstacle_avoidance(&group).is_landing_confirmation_needed);
    }

    #[test]
    fn obstacle_detection_sectors() {
        let group = read(&[
            0x01, 0x0e, 0x24, 0xe6, 0x09, 0x01, 0x27, 0x06, 0x01, 0x27, 0x06, 0x01, 0x10, 0x27,
            0x00, 0x81, 0xfe,
        ]);

        let VisionGroupEntry::ObstacleDetection(detection) = &group.entries[0] else {
            panic!("not an obstacle detection: {:?}", group.entries[0]);
        };
        assert_eq!(detection.direction, SensorDirection::Back);
        let distances: Vec<f32> = detection.sectors.iter().map(|s| s.distance).collect();
        assert_eq!(distances, [25.34, 15.75, 15.75, 100.0]);
        assert_eq!(detection.sectors[0].warning_level, 1);
    }
}
