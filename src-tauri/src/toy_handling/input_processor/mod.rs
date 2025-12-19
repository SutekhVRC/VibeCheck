use dyn_clone::DynClone;
use std::fmt::Debug;

use super::mode_processor::core::ModeProcessorInputType;

pub mod penetration_systems;

/*
 * Penetration System Architecture
 */

/*
 * Trait to define easily implementable behaviour for new penetration systems
 *
 */

pub trait InputProcessor: DynClone + Debug + Send + Sync {
    fn process(&mut self, addr: &str, input: ModeProcessorInputType) -> Option<f64>;
    fn is_parameter(&self, param: &str) -> bool;
    fn input_filter(&self, param: &str, filter: &Vec<String>) -> bool;
}
dyn_clone::clone_trait_object!(InputProcessor);
