use crate::spacial::transform::Transform;

crate::partition! {
    pub struct CoreAddition as CoreView {
        pub xforms: Class<Transform>,
        pub heirarchy: Class<Transform>,
        pub names: Class<String>,
    }
}
