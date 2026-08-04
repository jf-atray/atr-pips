use std::any::Any;

use glam::Vec3;

use crate::spacial::transform::Transform;

struct FishContext {
    pub color: f32,
}
impl Make for FishContext {
    fn make_into<'a>(&mut self, view: &'a mut TableView) -> &'a mut TableView {
        view.affirm_xyz(|xform| { xform.with_xyz(Vec3::Z); })
    }
}

pub trait Make: Any {
    fn make_into<'a>(&mut self, view: &'a mut TableView) -> &'a mut TableView;
}

pub struct TableView {
    xform: Option<Transform>,
}
impl TableView {
    pub fn affirm_xyz<F: FnOnce(&mut Transform)>(&mut self, f: F) -> &mut Self {
        let xform = self.xform.get_or_insert_with(Transform::default);
        f(xform);
        self
    }
}
pub struct TableMain {
    xforms: Class<Transform>,
}

pub struct Table {
    xform: Class<Transform>,
}

pub struct Class<T> {
    columns: Vec<ClassRow<T>>,
}
pub struct ClassRow<T> {
    rows: Vec<T>,
}
