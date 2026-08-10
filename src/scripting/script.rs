use std::any::Any;

use crate::scripting::scripts::Scripts;

pub trait Script: Any {
    fn update(&mut self, scripts: &Scripts);
}
