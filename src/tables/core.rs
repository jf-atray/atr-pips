use crate::brushes::Brush;
use crate::spacial::transform::Transform;

crate::partition! {
    pub struct CoreAddition as CoreView {
        pub xforms: Class<Transform, ()>,
        pub brushes: Class<Brush>,
        pub names: Class<String>,
    }
}
