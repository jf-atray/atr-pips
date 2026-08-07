pub mod class;
pub mod class_strategy;
pub mod core;
pub mod domain;
pub mod partition;
pub mod scope;
pub mod tables;
pub mod demo;

slotmap::new_key_type! {
    pub struct ClassId;
    //put these in canvassing mod
    pub struct CanvasId;
    pub struct CanvasSolverId;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ClassRowPtr {
    pub class_id: ClassId, //u64. pain in the arse to shrink but also lol memory.
    pub row_idx: usize,
}
impl ClassRowPtr {
    pub fn new(class_id: ClassId, row_idx: usize) -> Self {
        Self { class_id, row_idx }
    }
}