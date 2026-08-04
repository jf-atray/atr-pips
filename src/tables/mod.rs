pub mod class;
pub mod class_strategy;
pub mod make;

slotmap::new_key_type! {
    pub struct ClassId;
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