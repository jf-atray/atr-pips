use glam::Vec2;

//gamejam majic numbers. enter if you dare

pub const FAR_Z: f32 = 12.0;
pub const NEAR_Z: f32 = 1.0;
pub const GROUND_Y: f32 = 4.5;
pub const HORIZON_Y: f32 = 1.0;
pub const DESPAWN_T: f32 = 4.5;
pub const Z_WORLD_NEAR: f32 = 0.25;
pub const Z_WORLD_FAR: f32 = 0.75;
pub const WALK_SPEED: f32 = 0.66;

const BEHIND_D: f32 = -10.0;

pub fn depth_factor(d: f32) -> f32 {
    if d >= NEAR_Z {
        let inv_d = 1.0 / d;
        let inv_far = 1.0 / FAR_Z;
        let inv_near = 1.0 / NEAR_Z;
        (inv_d - inv_far) / (inv_near - inv_far)
    } else {
        let slope = -FAR_Z / (NEAR_Z * (FAR_Z - NEAR_Z));
        1.0 + slope * (d - NEAR_Z)
    }
}

pub fn depth_factor_linear(d: f32) -> f32 {
    let l = (FAR_Z - d) / (FAR_Z - NEAR_Z);
    if l < 1.0 && l > 0.0 {
        l * l
    } else {
        l
    }
}

pub fn world_x(dx: f32, t: f32) -> f32 {
    if t > 1.0 {
        return dx * t * t;
    }
    dx * t
}

pub fn world_y(t: f32, ground_y: f32, horizon_y: f32) -> f32 {
    
    if t <= 1.0 {
        let ratio = 0.25;
        let ratio_pi = std::f32::consts::PI * ratio;
        let a = (ratio_pi + (t * std::f32::consts::PI * (1.0 - ratio))).sin();
        (a * (ground_y + horizon_y)) - ground_y
    } else {
        let a = 1.0 - t * t * t * t;
        (a * (ground_y + horizon_y)) - ground_y
    }
}

pub fn projected_scale(t: f32) -> f32 {
    t
}

pub fn world_z(d: f32) -> f32 {
    let t = 1.0 - depth_factor_linear(d);
    Z_WORLD_NEAR + t * (Z_WORLD_FAR - Z_WORLD_NEAR)
}

pub fn project(
    lateral: f32,
    d: f32,
    player_lateral: f32,
    scalar: f32,
    ground_y: f32,
    horizon_y: f32,
) -> (Vec2, f32) {
    let t_linear = depth_factor_linear(d);
    let dx = lateral - player_lateral;
    let pos = Vec2::new(world_x(dx, t_linear), world_y(t_linear, ground_y, horizon_y));

    if scalar < 0.5 {
        let s = projected_scale((t_linear - 0.333) * (3.0 / 2.0)).max(0.0);
        (pos, s)
    } else {
        let s = projected_scale(t_linear);
        (pos, s)
    }
}
