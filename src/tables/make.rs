use std::any::Any;

use glam::{Vec3, prelude::Vec4};

use crate::{spacial::transform::Transform, tables::class::Class};

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
impl Make<TableView> for FishContext {
    fn make_into<'a>(&mut self, view: &'a mut TableView) -> &'a mut TableView {
        view.xform(|xform| xform.xyz(Vec3::Z))
    }
}
impl Make<TableViewColor> for FishContext {
    fn make_into<'a>(&mut self, view: &'a mut TableViewColor) -> &'a mut TableViewColor {
        view.color(|color| color.rgba(Vec4::ZERO))
    }
}

pub struct Tables {
    pub tables: Vec<dyn Viewable>,
}


pub trait Make<T>: Any {
    fn make_into<'a>(&mut self, view: &'a mut T) -> &'a mut T;
}

pub trait Viewable {
    type Out;
    fn view(&mut self) -> Self::Out;
}
pub struct TableMain {
    xforms: Class<Transform>,
}
pub struct TableView {
    xform: Option<Transform>,
}
impl Viewable for TableMain {
    type Out = TableView;

    fn view(&mut self) -> Self::Out {
        todo!()
    }
} 
impl Viewable for TableColor {
    type Out = TableViewColor;

    fn view(&mut self) -> Self::Out {
        todo!()
    }
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

