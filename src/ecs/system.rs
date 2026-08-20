use crate::ecs::PipId;

crate::partition! {
    pub struct SystemAddition as SystemView {
        pub pip_id: Class<PipId>,
    }
}
