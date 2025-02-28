use std::sync::Arc;

use buttplug::client::ButtplugClientDevice;
use rosc::{OscMessage, OscType};

use crate::toy_handling::toyops::ToyParameter;
use crate::toy_handling::toyops::VCToyFeatures;

use log::info;

use super::mode_processor;
use super::mode_processor::core::ModeProcessorInput;
use super::mode_processor::core::ModeProcessorInputType;
use super::toy_command_processor::command_toy;
use super::toyops::ProcessingMode;
use mode_processor::core::mode_processor;

#[inline(always)]
pub async fn parse_osc_message(
    msg: &mut OscMessage,
    dev: Arc<ButtplugClientDevice>,
    vc_toy_features: &mut VCToyFeatures,
) {
    // Parse OSC msgs to toys commands
    //debug!("msg.addr = {} | msg.args = {:?}", msg.addr, msg.args);
    /*
     * Do Penetration System parsing first?
     * Then parameter parsing?
     * Mode processor is a function now so it can be used in both!
     */

    // msg args pop should go here
    //let newest_msg_addr = msg.addr.clone();
    let newest_msg_val = msg.args.pop().unwrap();

    /*
     * Input mode processing
     * Get all features with an enabled Input mode
     * For each feature with a penetration system do processing for the current OSC input
     * If get a value from processing check if the Input mode has a processing mode associated
     *
     */

    // Get all features with an enabled input processor?
    if let Some(input_processor_system_features) =
        vc_toy_features.get_features_with_input_processors(&msg.addr)
    {
        match newest_msg_val {
            OscType::Float(lvl) => {
                for feature in input_processor_system_features {
                    let float_level = ((lvl * 100.0).round() / 100.0) as f64;
                    // pen_system is checked for None in get_features_with_penetration_systems method.
                    // Give access to internal mode values here (input, internal_values)
                    if let Some(i_mode_processed_value) = feature
                        .penetration_system
                        .pen_system
                        .as_mut()
                        .unwrap()
                        .process(
                            msg.addr.as_str(),
                            ModeProcessorInputType::Float(float_level),
                        )
                    {
                        // Send to mode processor if specified (Raw = no mode processing)
                        if let ProcessingMode::Raw =
                            feature.penetration_system.pen_system_processing_mode
                        {
                            command_toy(
                                dev.clone(),
                                feature.feature_type,
                                i_mode_processed_value,
                                feature.feature_index,
                                feature.flip_input_float,
                                feature.feature_levels,
                            )
                            .await;
                        } else {
                            // If mode processor returns a value send to toy
                            if let Some(i) = mode_processor(
                                ModeProcessorInput::InputProcessor((
                                    ModeProcessorInputType::Float(i_mode_processed_value),
                                    &mut feature
                                        .penetration_system
                                        .pen_system_processing_mode_values,
                                )),
                                feature.feature_levels,
                                feature.flip_input_float,
                            )
                            .await
                            {
                                command_toy(
                                    dev.clone(),
                                    feature.feature_type,
                                    i,
                                    feature.feature_index,
                                    feature.flip_input_float,
                                    feature.feature_levels,
                                )
                                .await;
                            }
                        }
                    }
                }
            }
            // Boolean can be supported in the process trait method
            OscType::Bool(b) => {
                for feature in input_processor_system_features {
                    // Boolean to float transformation here
                    if let Some(i_mode_processed_value) = feature
                        .penetration_system
                        .pen_system
                        .as_mut()
                        .unwrap()
                        .process(msg.addr.as_str(), ModeProcessorInputType::Boolean(b))
                    {
                        // Send to mode processor if specified (Raw = no mode processing)
                        if let ProcessingMode::Raw =
                            feature.penetration_system.pen_system_processing_mode
                        {
                            command_toy(
                                dev.clone(),
                                feature.feature_type,
                                i_mode_processed_value,
                                feature.feature_index,
                                feature.flip_input_float,
                                feature.feature_levels,
                            )
                            .await;
                        } else if let Some(i) = mode_processor(
                            ModeProcessorInput::InputProcessor((
                                ModeProcessorInputType::Float(i_mode_processed_value),
                                &mut feature.penetration_system.pen_system_processing_mode_values,
                            )),
                            feature.feature_levels,
                            feature.flip_input_float,
                        )
                        .await
                        {
                            command_toy(
                                dev.clone(),
                                feature.feature_type,
                                i,
                                feature.feature_index,
                                feature.flip_input_float,
                                feature.feature_levels,
                            )
                            .await;
                        }
                    }
                }
            }
            _ => (),
        } // End match OscType for Input processors
    } // End Input processing

    if let Some(features) = vc_toy_features.get_features_from_param(&msg.addr) {
        match newest_msg_val {
            OscType::Float(lvl) => {
                // Clamp float accuracy to hundredths and cast as 64 bit float
                let float_level = ((lvl * 100.0).round() / 100.0) as f64;
                //debug!("Received and cast float lvl: {:.5}", float_level);

                for feature in features {
                    // Get ToyParameter here
                    // We unwrap here because the call to get_features_from_param guarantees the parameter exists.
                    let mut toy_parameter = feature
                        .osc_parameters
                        .iter_mut()
                        .filter_map(|param| {
                            if param.parameter == msg.addr {
                                Some(param)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<&mut ToyParameter>>();

                    if let Some(first_toy_param) = toy_parameter.first_mut() {
                        if let Some(mode_processed_value) = mode_processor(
                            ModeProcessorInput::RawInput(
                                ModeProcessorInputType::Float(float_level),
                                first_toy_param,
                            ),
                            feature.feature_levels,
                            feature.flip_input_float,
                        )
                        .await
                        {
                            command_toy(
                                dev.clone(),
                                feature.feature_type,
                                mode_processed_value,
                                feature.feature_index,
                                feature.flip_input_float,
                                feature.feature_levels,
                            )
                            .await;
                        }
                    } // If no matching toy parameter skip feature
                }
            }
            OscType::Bool(b) => {
                info!("Got a Bool! {} = {}", msg.addr, b);
                for feature in features {
                    // Get ToyParameter here
                    let mut toy_parameter = feature
                        .osc_parameters
                        .iter_mut()
                        .filter_map(|param| {
                            if param.parameter == msg.addr {
                                Some(param)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<&mut ToyParameter>>();

                    if let Some(first_toy_param) = toy_parameter.first_mut() {
                        if let Some(i) = mode_processor(
                            ModeProcessorInput::RawInput(
                                ModeProcessorInputType::Boolean(b),
                                first_toy_param,
                            ),
                            feature.feature_levels,
                            feature.flip_input_float,
                        )
                        .await
                        {
                            command_toy(
                                dev.clone(),
                                feature.feature_type,
                                i,
                                feature.feature_index,
                                feature.flip_input_float,
                                feature.feature_levels,
                            )
                            .await;
                        }
                    }
                }
            }
            _ => {} // Skip parameter because unsuppported OSC type
        }
    }
}
