use std::any::Any;

use crate::scripting::context::DomainView;

pub trait Script: Any + std::fmt::Debug {
    fn update(&mut self, ctx: &mut DomainView);
}
