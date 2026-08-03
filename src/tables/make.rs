use std::any::Any;

pub trait Make: Any {
    fn make(view: TableView) -> TableHand;
}

pub struct TableView<'table> {
    xform: Option<&'table mut Vec<f32>>,
}
pub struct TableHand<'table> {
    xform: Option<Hand<'table, f32>>,
}
pub struct Hand<'table, T> {
    pub class: &'table mut Vec<T>,
    pub datum: T,
}