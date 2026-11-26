use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::vision_group::{
    ObstacleAvoidance, ObstacleDetection, SensorDirection, VisionGroupEntry,
};

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameVision {
    /// Indicates if obstacle avoidance is enabled
    pub is_obstacle_avoidance_enabled: bool,
    /// Indicates if collision avoidance is enabled
    pub is_collision_avoidance_enabled: bool,
    /// Indicates if the aircraft is avoiding an obstacle, on older aircraft
    pub is_avoiding_obstacle_legacy: bool,
    /// Indicates if the aircraft is braking in front of an obstacle
    pub is_braking: bool,
    /// Indicates if obstacle avoidance is enabled during return to home
    pub is_rth_obstacle_avoidance_enabled: bool,
    /// Indicates if the landing must be confirmed by the pilot
    pub is_landing_confirmation_needed: bool,
    /// Indicates if the aircraft is at its distance limit
    pub is_at_distance_limit: bool,
    /// Indicates if the aircraft is at the boundary of an airport zone
    pub is_at_airport_boundary: bool,
    /// Indicates if the aircraft is avoiding an obstacle
    pub is_avoiding_obstacle: bool,
    /// Indicates if the aircraft is near its horizontal limit
    pub is_near_horizontal_limit: bool,
    /// Indicates if the aircraft is avoiding an obstacle moving toward it
    pub is_avoiding_active_obstacle: bool,
    /// Indicates if the aircraft is at its low altitude limit
    pub is_at_low_altitude_limit: bool,
    /// Indicates if the aircraft is at the altitude limit of an airport zone
    pub is_at_airport_altitude_limit: bool,
    /// Indicates if the ascent is limited by an obstacle above the aircraft
    pub is_ascent_limited_by_obstacle: bool,
    /// Indicates if the aircraft is at its limit above the ground
    pub is_at_ground_limit: bool,
    /// Distance of the closest obstacle in front of the aircraft, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub front_distance: Option<f32>,
    /// Distance of the closest obstacle behind the aircraft, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub back_distance: Option<f32>,
    /// Distance of the closest obstacle on the right of the aircraft, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub right_distance: Option<f32>,
    /// Distance of the closest obstacle on the left of the aircraft, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub left_distance: Option<f32>,
    /// Distance of the closest obstacle above the aircraft, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub top_distance: Option<f32>,
    /// Distance of the closest obstacle below the aircraft, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub bottom_distance: Option<f32>,
}

/// Distance reported by a sector when it detects nothing
const NO_OBSTACLE_DISTANCE: f32 = 100.0;

impl FrameVision {
    pub(crate) fn update(&mut self, entry: &VisionGroupEntry) {
        match entry {
            VisionGroupEntry::ObstacleDetection(detection) => self.update_detection(detection),
            VisionGroupEntry::ObstacleAvoidance(avoidance) => self.update_avoidance(avoidance),
            VisionGroupEntry::Unknown { .. } => {}
        }
    }

    fn update_detection(&mut self, detection: &ObstacleDetection) {
        let distance = detection
            .sectors
            .iter()
            .map(|sector| sector.distance)
            .filter(|distance| *distance < NO_OBSTACLE_DISTANCE)
            .min_by(f32::total_cmp);

        match detection.direction {
            SensorDirection::Front => self.front_distance = distance,
            SensorDirection::Back => self.back_distance = distance,
            SensorDirection::Right => self.right_distance = distance,
            SensorDirection::Left => self.left_distance = distance,
            SensorDirection::Top => self.top_distance = distance,
            SensorDirection::Bottom => self.bottom_distance = distance,
            SensorDirection::Unknown(_) => {}
        }
    }

    fn update_avoidance(&mut self, avoidance: &ObstacleAvoidance) {
        self.is_obstacle_avoidance_enabled = avoidance.is_obstacle_avoidance_enabled;
        self.is_collision_avoidance_enabled = avoidance.is_collision_avoidance_enabled;
        self.is_avoiding_obstacle_legacy = avoidance.is_avoiding_obstacle_legacy;
        self.is_braking = avoidance.is_braking;
        self.is_rth_obstacle_avoidance_enabled = avoidance.is_rth_obstacle_avoidance_enabled;
        self.is_landing_confirmation_needed = avoidance.is_landing_confirmation_needed;
        self.is_at_distance_limit = avoidance.is_at_distance_limit;
        self.is_at_airport_boundary = avoidance.is_at_airport_boundary;
        self.is_avoiding_obstacle = avoidance.is_avoiding_obstacle;
        self.is_near_horizontal_limit = avoidance.is_near_horizontal_limit;
        self.is_avoiding_active_obstacle = avoidance.is_avoiding_active_obstacle;
        self.is_at_low_altitude_limit = avoidance.is_at_low_altitude_limit;
        self.is_at_airport_altitude_limit = avoidance.is_at_airport_altitude_limit;
        self.is_ascent_limited_by_obstacle = avoidance.is_ascent_limited_by_obstacle;
        self.is_at_ground_limit = avoidance.is_at_ground_limit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::vision_group::ObstacleSector;

    fn detection(direction: SensorDirection, distances: &[f32]) -> VisionGroupEntry {
        VisionGroupEntry::ObstacleDetection(ObstacleDetection {
            direction,
            sectors: distances
                .iter()
                .map(|&distance| ObstacleSector {
                    distance,
                    warning_level: 0,
                })
                .collect(),
        })
    }

    #[test]
    fn distance_is_the_closest_sector_with_an_obstacle() {
        let mut vision = FrameVision::default();

        vision.update(&detection(
            SensorDirection::Back,
            &[100.0, 3.32, 15.75, 100.0],
        ));
        vision.update(&detection(
            SensorDirection::Front,
            &[100.0, 100.0, 100.0, 100.0],
        ));

        assert_eq!(vision.back_distance, Some(3.32));
        assert_eq!(vision.front_distance, None);
    }
}
