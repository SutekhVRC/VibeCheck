use crate::{
    frontend::frontend_types::{
        FeLevelTweaks, FePenetrationSystem, FeProcessingMode, FeToyParameter, FeVCFeatureType,
        FeVCToy, FeVCToyAnatomy, FeVCToyFeature,
    },
    toy_handling::input_processor::penetration_systems::PenetrationSystemType,
    toy_handling::ToyPower,
};

fn default_levels() -> FeLevelTweaks {
    FeLevelTweaks {
        minimum_level: 0.0,
        maximum_level: 1.0,
        idle_level: 0.0,
        smooth_rate: 0.0,
        linear_position_speed: 500,
        rate_tune: 0.0,
        constant_level: 0.0,
    }
}

fn vibrator_feature(feature_index: u32, parameter: &str) -> FeVCToyFeature {
    FeVCToyFeature {
        feature_enabled: true,
        feature_type: FeVCFeatureType::Vibrator,
        osc_parameters: vec![FeToyParameter {
            parameter: parameter.to_string(),
            processing_mode: FeProcessingMode::Raw,
        }],
        penetration_system: FePenetrationSystem {
            pen_system_type: PenetrationSystemType::None,
            pen_system_processing_mode: FeProcessingMode::Raw,
        },
        feature_index,
        flip_input_float: false,
        feature_levels: default_levels(),
        smooth_enabled: false,
        rate_enabled: false,
    }
}

pub fn mock_toys() -> Vec<FeVCToy> {
    vec![
        FeVCToy {
            toy_id: Some(1),
            toy_name: "Mock Vibrator".to_string(),
            toy_anatomy: FeVCToyAnatomy::Clitoris,
            toy_power: ToyPower::Battery(0.82),
            toy_connected: true,
            features: vec![
                vibrator_feature(0, "/avatar/parameters/MockFeature1"),
                vibrator_feature(1, "/avatar/parameters/MockFeature2"),
            ],
            listening: true,
            osc_data: true,
            bt_update_rate: 20,
            sub_id: 0,
        },
        FeVCToy {
            toy_id: Some(2),
            toy_name: "Mock Rotator".to_string(),
            toy_anatomy: FeVCToyAnatomy::Vagina,
            toy_power: ToyPower::NoBattery,
            toy_connected: true,
            features: vec![vibrator_feature(0, "/avatar/parameters/MockRotation")],
            listening: true,
            osc_data: false,
            bt_update_rate: 20,
            sub_id: 1,
        },
    ]
}
