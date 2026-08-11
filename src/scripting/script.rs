use std::any::Any;

use crate::scripting::context::DomainView;

pub trait Script: Any {
    fn update(&mut self, ctx: &DomainView);
}
