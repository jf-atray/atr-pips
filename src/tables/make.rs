use std::{any::{Any, TypeId}, collections::HashMap};

use glam::{Vec3, prelude::Vec4};
use slotmap::{SlotMap, new_key_type};

use crate::{spacial::transform::Transform, tables::{class::Class, class_strategy::rarity}};

#[derive(Default, Debug)]
pub struct Color {
    pub rgba: Vec4,
}
impl Color {
    pub fn rgba(&mut self, rgba: Vec4) -> &mut Self {
        self.rgba = rgba;
        self
    }
}
struct FishContext {
    pub color: f32,
}
impl FishContext {
    pub fn make_into(core: &mut CoreAddition, additions: AdditionsView) {
        
    }
}
new_key_type! {
    pub struct TableIndirectionId;
}
struct CoreAddition {
    pub xforms: Class<Transform>,
    pub heirarchy: Class<Transform>,
    pub names: Class<String>,
}
pub trait Addition: Any {}
impl Addition for CoreAddition {}
pub struct Domain {
    core: CoreAddition,
    additions: HashMap<TypeId, Box<dyn Addition>>,
}
pub struct AdditionsView<'domain> {
    additions: &'domain mut HashMap<TypeId, Box<dyn Addition>>,
}

impl<'domain> AdditionsView<'domain> {
    pub fn get_addition<T: Addition + 'static>(&mut self) -> Option<&mut T> {
        let id = TypeId::of::<T>();
        self.additions
            .get_mut(&id)
            .and_then(|any| (any.as_mut() as &mut dyn Any).downcast_mut::<T>())
    }
}
pub struct TableMain {
    xforms: Class<Transform>,
}
pub struct TableView {
    xform: Option<Transform>,
}
impl TableView {
    
}
struct World {
    view: TableView,
    color: TableViewColor,
}

impl TableView {
    pub fn xform<F: FnOnce(&mut Transform) -> &mut Transform>(&mut self, f: F) -> &mut Self {
        let xform = self.xform.get_or_insert_with(Transform::default);
        f(xform);
        self
    }
}

pub struct TableColor {
    xforms: Class<Color>,
}
pub struct TableViewColor {
    xform: Option<Color>,
}
impl TableViewColor {
    pub fn color<F: FnOnce(&mut Color) -> &mut Color>(&mut self, f: F) -> &mut Self {
        let xform = self.xform.get_or_insert_with(Color::default);
        f(xform);
        self
    }
}


