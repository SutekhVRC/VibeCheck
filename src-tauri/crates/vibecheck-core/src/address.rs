//! Stable, typed addresses for module ports.
//!
//! Addresses must survive restarts and reconnects, so they never contain
//! Buttplug device indexes. Toys are identified the way the toy config does it
//! (name plus duplicate id), and output modules map that to a device index
//! internally.

use std::fmt;
use std::sync::Arc;

/// Identifies one instance of a module, e.g. `osc`, `toy:lovense_lush:0` or `output:0`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleInstanceId(Arc<str>);

impl ModuleInstanceId {
    pub fn new(id: impl AsRef<str>) -> Self {
        Self(Arc::from(id.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One named port on a module instance, e.g. `toy:lovense_lush:0/vibrate_0`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortId {
    pub module: ModuleInstanceId,
    pub port: Arc<str>,
}

impl PortId {
    pub fn new(module: ModuleInstanceId, port: impl AsRef<str>) -> Self {
        Self {
            module,
            port: Arc::from(port.as_ref()),
        }
    }
}

impl fmt::Display for PortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.module, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_as_module_slash_port() {
        let port = PortId::new(ModuleInstanceId::new("toy:lovense_lush:0"), "vibrate_0");
        assert_eq!(port.to_string(), "toy:lovense_lush:0/vibrate_0");
    }
}
