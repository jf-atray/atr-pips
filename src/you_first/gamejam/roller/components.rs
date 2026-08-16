#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RollerDepth {
    pub d: f32,
    pub lateral: f32,
    pub speed: f32,
    pub scalar: f32,
    pub lateral_speed: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RollerPlayer {
    pub walk_distance: f32,
    pub lateral: f32,
}

crate::partition! {
    pub struct RollerAddition as RollerView {
        pub roller_depths: Class<RollerDepth>,
        pub roller_players: Class<RollerPlayer>,
    }
}
