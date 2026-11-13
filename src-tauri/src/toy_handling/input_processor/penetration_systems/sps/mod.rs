/*
 * SPS input processor.
 *
 * SPS (https://vrcfury.com/sps/) reports contact through avatar parameters
 * under OGB/ (and VFH/Zone/ from newer VRCFury), one set per zone: a
 * socket, plug or touch area. See zone.rs for the address format and how
 * each zone's readings are worked out, for both the legacy format (Pen/Orf)
 * and SPS3 (Plug/Socket).
 *
 * Every zone gives several readings: depth from the avatar's own body and
 * from others, touch from either, and frot. The feature's level is the
 * strongest reading of any zone that passes the input filter. Features of
 * Linear type only follow depth.
 *
 * The input filter matches a tag against the zone's name (e.g. a socket's
 * menu label): a zone counts if its name contains any tag.
 *
 * If the avatar sets OGB/HapticsDisabled, the level is 0 until it's
 * cleared.
 */

mod length;
mod zone;

use std::collections::HashMap;

use vibecheck_core::Signal;

use crate::toy_handling::input_processor::InputProcessor;

use self::zone::{Address, Zone};

const PARAMS: &str = "/avatar/parameters/";
const HAPTICS_DISABLED: &str = "OGB/HapticsDisabled";

/// Whether `addr` is an SPS zone setting (a plug's length, a socket's
/// radius), which VRChat may never send over OSC.
pub fn is_setting_address(addr: &str) -> bool {
    addr.strip_prefix(PARAMS)
        .and_then(Address::parse)
        .is_some_and(|a| zone::is_setting(a.contact))
}

pub struct SPSProcessor {
    /// Whether the feature is Linear (depth only).
    linear: bool,
    zones: HashMap<String, Zone>,
    haptics_disabled: bool,
    /// Whether any zone has reported contact yet (settings such as a plug's
    /// length don't count).
    active: bool,
}

impl SPSProcessor {
    pub fn new(linear: bool) -> Self {
        Self {
            linear,
            zones: HashMap::new(),
            haptics_disabled: false,
            active: false,
        }
    }
}

impl InputProcessor for SPSProcessor {
    fn prefixes(&self) -> Vec<String> {
        vec![format!("{PARAMS}OGB/"), format!("{PARAMS}VFH/Zone/")]
    }

    fn update(&mut self, addr: &str, value: Signal) {
        let Some(param) = addr.strip_prefix(PARAMS) else {
            return;
        };
        if param == HAPTICS_DISABLED {
            self.haptics_disabled = match value {
                Signal::Bool(b) => b,
                Signal::Float(v) => v > 0.0,
            };
            return;
        }
        let Some(address) = Address::parse(param) else {
            return; // Metadata such as OGB/Version/11
        };
        self.zones
            .entry(address.zone.to_owned())
            .or_insert_with(|| Zone::new(address.kind, address.format))
            .set(address.contact, value);
        // Settings arrive when the avatar loads; Idle Speed waits for contact
        if !zone::is_setting(address.contact) {
            self.active = true;
        }
    }

    fn tick(&mut self, _dt: f32) -> Option<f64> {
        if !self.active {
            return None;
        }
        for zone in self.zones.values_mut() {
            zone.settle();
        }
        if self.haptics_disabled {
            return Some(0.0);
        }
        let strongest = self
            .zones
            .values()
            .flat_map(|zone| zone.readings(self.linear))
            .fold(0.0, f64::max);
        Some(strongest.clamp(0.0, 1.0))
    }

    fn reset(&mut self) {
        self.zones.clear();
        self.haptics_disabled = false;
        self.active = false;
    }

    /// Tags match the zone's name. Anything that isn't a zone contact
    /// (OGB/HapticsDisabled, version markers) always gets through.
    fn input_filter(&self, addr: &str, tags: &[String]) -> bool {
        let zone = addr.strip_prefix(PARAMS).and_then(Address::parse);
        match zone {
            Some(zone) => tags.iter().any(|tag| zone.id.contains(tag.as_str())),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Avatar(SPSProcessor);

    impl Avatar {
        fn new() -> Self {
            Self(SPSProcessor::new(false))
        }

        fn linear() -> Self {
            Self(SPSProcessor::new(true))
        }

        fn float(&mut self, param: &str, value: f64) -> &mut Self {
            self.0
                .update(&format!("{PARAMS}{param}"), Signal::Float(value));
            self
        }

        fn bool(&mut self, param: &str, value: bool) -> &mut Self {
            self.0
                .update(&format!("{PARAMS}{param}"), Signal::Bool(value));
            self
        }

        fn level(&mut self) -> f64 {
            self.0.tick(0.01).expect("a level")
        }
    }

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} is not {b}");
    }

    #[test]
    fn nothing_until_a_zone_sends_something() {
        let mut a = Avatar::new();
        assert_eq!(a.0.tick(0.01), None);
        a.bool("OGB/Version/11", true);
        a.bool(HAPTICS_DISABLED, false);
        assert_eq!(a.0.tick(0.01), None, "metadata isn't a zone");
        a.float("OGB/Orf/Mouth/PenOthers", 0.0);
        assert_eq!(a.0.tick(0.01), Some(0.0));
    }

    #[test]
    fn settings_alone_dont_start_anything() {
        let mut a = Avatar::new();
        // What the OSCQuery lookup fills in when the avatar loads
        a.float("OGB/Plug/Dp/LengthMult1", 0.0112)
            .float("OGB/Plug/Dp/LengthMult2", 17.37)
            .float("OGB/Socket/Hand_R/DepthRadius", 1.0);
        assert_eq!(a.0.tick(0.01), None, "Idle Speed waits for contact");
        a.float("OGB/Plug/Dp/DepthSelf", 0.3);
        assert!(a.0.tick(0.01).is_some());

        assert!(is_setting_address(
            "/avatar/parameters/OGB/Plug/Dp/LengthMult2"
        ));
        assert!(is_setting_address(
            "/avatar/parameters/VFH/Zone/Socket/Hand_R/DepthRadius"
        ));
        assert!(!is_setting_address(
            "/avatar/parameters/OGB/Plug/Dp/DepthSelf"
        ));
        assert!(!is_setting_address("/avatar/parameters/LengthMult2"));
    }

    /// Values captured from a real SPS3 plug and hand socket in VRChat
    /// (2026-11-12). LengthMult2 was never sent over OSC; its value is from
    /// VRChat's OSCQuery.
    #[test]
    fn sps3_plug_and_hand_socket_from_a_real_capture() {
        let mut a = Avatar::new();
        a.float("OGB/Plug/zw_ris_sps/LengthMult1", 0.011209071)
            .float("OGB/Plug/zw_ris_sps/DepthSelf", 0.3255276)
            .float("OGB/Plug/zw_ris_sps/DepthSelf_PointSeparation", 0.21013895);
        // Without LengthMult2 there's no length, so no depth
        near(a.level(), 0.0);

        // The plug is 0.195 m long and the hand 0.21 m from its base: just
        // past the tip, so still no depth
        a.float("OGB/Plug/zw_ris_sps/LengthMult2", 17.3739777);
        near(a.level(), 0.0);
        // Halfway down
        a.float("OGB/Plug/zw_ris_sps/DepthSelf_PointSeparation", 0.0974);
        assert!((a.level() - 0.5).abs() < 0.01);

        // The hand socket's own estimate of the plug's length from its
        // radius agrees: 0.2126 - 1.0 × (1 - 0.9821) = 0.195 m
        let mut a = Avatar::new();
        a.float("OGB/Socket/Hand_R/DepthRadius", 1.0)
            .float("OGB/Socket/Hand_R/DepthSelf", 0.9821344)
            .float("OGB/Socket/Hand_R/DepthSelf_PointSeparation", 0.21259077);
        near(a.level(), 0.0);
        a.float("OGB/Socket/Hand_R/DepthSelf", 1.0)
            .float("OGB/Socket/Hand_R/DepthSelf_PointSeparation", 0.0974);
        assert!((a.level() - 0.5).abs() < 0.01);
    }

    #[test]
    fn sps3_plug_depth_uses_its_own_length() {
        let mut a = Avatar::new();
        // 0.01 × 40 = 0.4 m long, 0.1 m of it still outside
        a.float("OGB/Plug/Dp/LengthMult1", 0.01)
            .float("OGB/Plug/Dp/LengthMult2", 40.0)
            .float("OGB/Plug/Dp/DepthOthers", 1.0)
            .float("OGB/Plug/Dp/DepthOthers_PointSeparation", 0.1);
        near(a.level(), 0.75);
        a.float("OGB/Plug/Dp/DepthOthers_PointSeparation", 0.0);
        near(a.level(), 1.0);
        a.float("OGB/Plug/Dp/DepthOthers_PointSeparation", 0.6);
        near(a.level(), 0.0);
        // No length, no depth
        a.float("OGB/Plug/Dp/DepthOthers_PointSeparation", 0.1)
            .float("OGB/Plug/Dp/LengthMult1", 0.0);
        near(a.level(), 0.0);
    }

    #[test]
    fn sps3_socket_uses_the_length_a_native_plug_reports() {
        let mut a = Avatar::new();
        a.float("OGB/Socket/Mouth/DepthSelf", 1.0)
            .float("OGB/Socket/Mouth/DepthSelf_PointSeparation", 0.1)
            .float("OGB/Socket/Mouth/DepthSelf_SenderLossyScaleZ", 0.4)
            .bool(
                "OGB/Socket/Mouth/DepthSelf_CollisionTag_SPS/Plug/_public",
                true,
            );
        near(a.level(), 0.75);
        a.float("OGB/Socket/Mouth/DepthSelf_SenderLossyScaleZ", 0.8);
        near(a.level(), 0.875);
        a.float("OGB/Socket/Mouth/DepthSelf", 0.0);
        near(a.level(), 0.0);
    }

    #[test]
    fn sps3_socket_estimates_a_legacy_plug_and_keeps_it_while_inside() {
        let mut a = Avatar::new();
        // Partway in: 0.5 - 0.2 × (1 - 0.5) = 0.4 m long
        a.float("VFH/Zone/Socket/Mouth/DepthRadius", 0.2)
            .float("VFH/Zone/Socket/Mouth/DepthOthers", 0.5)
            .float("VFH/Zone/Socket/Mouth/DepthOthers_PointSeparation", 0.5)
            // Without a collision tag, the sender's scale isn't a length
            .float("VFH/Zone/Socket/Mouth/DepthOthers_SenderLossyScaleZ", 99.0);
        near(a.level(), 0.0);
        // Fully in: the estimate is kept rather than shrunk
        a.float("VFH/Zone/Socket/Mouth/DepthOthers", 1.0)
            .float("VFH/Zone/Socket/Mouth/DepthOthers_PointSeparation", 0.2);
        near(a.level(), 0.5);
        a.float("VFH/Zone/Socket/Mouth/DepthOthers_PointSeparation", 0.1);
        near(a.level(), 0.75);
    }

    #[test]
    fn legacy_socket_learns_the_plug_length_from_root_and_tip() {
        let mut a = Avatar::new();
        let zone = "OGB/Orf/Mouth/";
        // Approaching: four steady readings of a 0.3 m plug
        for root in [0.1, 0.2, 0.3, 0.4] {
            a.float(&format!("{zone}PenOthersNewRoot"), root)
                .float(&format!("{zone}PenOthersNewTip"), root + 0.3);
            near(a.level(), 0.0); // the tip isn't in yet
        }
        // Tip in, root at 0.85: 0.15 m of 0.3 m left outside
        a.float(&format!("{zone}PenOthersNewRoot"), 0.85)
            .float(&format!("{zone}PenOthersNewTip"), 1.0);
        near(a.level(), 0.5);
    }

    #[test]
    fn legacy_depth_and_touch_respect_close_flags() {
        let mut a = Avatar::new();
        a.float("OGB/Orf/Mouth/PenOthers", 0.6);
        near(a.level(), 0.6); // no PenOthersClose sent: counts
        a.bool("OGB/Orf/Mouth/PenOthersClose", false);
        near(a.level(), 0.0);

        let mut a = Avatar::new();
        a.float("OGB/Pen/Dp/TouchOthers", 0.7);
        near(a.level(), 0.0); // touch needs its Close flag
        a.bool("OGB/Pen/Dp/TouchOthersClose", true);
        near(a.level(), 0.7);

        // Frot on a legacy socket isn't gated
        let mut a = Avatar::new();
        a.float("OGB/Orf/Mouth/FrotOthers", 0.4);
        near(a.level(), 0.4);
    }

    #[test]
    fn touch_speed_replaces_proximity_when_sent() {
        let mut a = Avatar::new();
        a.float("OGB/Touch/Head/Others", 0.9)
            .float("OGB/Touch/Head/Others_SenderVelocityMagnitude", 0.4);
        near(a.level(), 0.4);
        a.float("OGB/Touch/Head/Others_SenderVelocityMagnitude", 0.0);
        near(a.level(), 0.0);
        a.float("OGB/Touch/Head/Others_SenderVelocityMagnitude", f64::NAN);
        near(a.level(), 0.0);
        // Fast, but not touching
        a.float("OGB/Touch/Head/Others_SenderVelocityMagnitude", 3.0)
            .float("OGB/Touch/Head/Others", 0.0);
        near(a.level(), 0.0);
        // Faster than 1 m/s is full power
        a.float("OGB/Touch/Head/Others", 0.5);
        near(a.level(), 1.0);
    }

    #[test]
    fn strongest_reading_of_any_zone_wins() {
        let mut a = Avatar::new();
        a.float("OGB/Orf/Mouth/PenOthers", 0.3)
            .float("OGB/Orf/Hips/PenOthers", 0.8)
            .float("OGB/Orf/Hips/PenSelf", 0.5);
        near(a.level(), 0.8);
    }

    #[test]
    fn linear_features_only_follow_depth() {
        let mut a = Avatar::linear();
        a.float("OGB/Touch/Head/Others", 0.9)
            .float("OGB/Orf/Mouth/FrotOthers", 0.9)
            .float("OGB/Orf/Mouth/PenOthers", 0.3);
        near(a.level(), 0.3);
    }

    #[test]
    fn haptics_disabled_silences_everything_until_cleared() {
        let mut a = Avatar::new();
        a.float("OGB/Orf/Mouth/PenOthers", 0.6);
        a.bool(HAPTICS_DISABLED, true);
        near(a.level(), 0.0);
        a.float(HAPTICS_DISABLED, 0.0);
        near(a.level(), 0.6);
    }

    #[test]
    fn input_filter_matches_zone_names_and_lets_metadata_through() {
        let p = SPSProcessor::new(false);
        let tags = vec!["Mouth".to_owned()];
        let pass = |param: &str| p.input_filter(&format!("{PARAMS}{param}"), &tags);
        assert!(pass("OGB/Orf/Mouth/PenOthers"));
        assert!(pass("OGB/Socket/MouthLower/DepthOthers"));
        assert!(!pass("OGB/Orf/Hips/PenOthers"));
        // The tag appears in the contact, not the zone name
        assert!(!pass(
            "OGB/Socket/Hips/DepthOthers_CollisionTag_SPS/Plug/Mouth"
        ));
        assert!(pass(HAPTICS_DISABLED));
        assert!(pass("OGB/Version/11"));
    }

    #[test]
    fn reset_forgets_zones_and_flags() {
        let mut a = Avatar::new();
        a.float("OGB/Orf/Mouth/PenOthers", 0.6);
        a.bool(HAPTICS_DISABLED, true);
        a.level();
        a.0.reset();
        assert_eq!(a.0.tick(0.01), None);
        a.float("OGB/Orf/Hips/PenOthers", 0.2);
        near(a.level(), 0.2);
    }
}
