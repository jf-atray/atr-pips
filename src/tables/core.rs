use crate::spacial::transform::Transform;

crate::partition! {
    pub struct CoreAddition as CoreView {
        pub xforms: Class<Vec<Transform>>,
        pub heirarchy: Class<Vec<Transform>>,
        pub names: Class<Vec<String>>,
    }
}
