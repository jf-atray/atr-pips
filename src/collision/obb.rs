use glam::{Quat, Vec2, Vec3};

pub struct Obb {
    pub center: Vec2,
    pub axes: [Vec2; 2],
    pub half_extents: Vec2,
}

impl Obb {
    pub fn from_transform(xyz: Vec3, rot: Quat, scale: Vec3) -> Self {
        let axis_x = rot * Vec3::X;
        let axis_y = rot * Vec3::Y;
        Self {
            center: Vec2::new(xyz.x, xyz.y),
            axes: [Vec2::new(axis_x.x, axis_x.y), Vec2::new(axis_y.x, axis_y.y)],
            half_extents: Vec2::new(scale.x, scale.y) * 0.5,
        }
    }

    fn corners(&self) -> [Vec2; 4] {
        let h = self.half_extents;
        let x = self.axes[0] * h.x;
        let y = self.axes[1] * h.y;
        [
            self.center - x - y,
            self.center + x - y,
            self.center + x + y,
            self.center - x + y,
        ]
    }

    fn edge_normal(&self, edge: usize) -> Vec2 {
        match edge {
            0 => -self.axes[1],
            1 => self.axes[0],
            2 => self.axes[1],
            3 => -self.axes[0],
            _ => unreachable!(),
        }
    }

    fn project(&self, axis: Vec2) -> (f32, f32) {
        let c = self.center.dot(axis);
        let r = self.half_extents.x * self.axes[0].dot(axis).abs()
            + self.half_extents.y * self.axes[1].dot(axis).abs();
        (c - r, c + r)
    }
}

pub struct SatResult {
    pub normal: Vec2,
    pub penetration: f32,
    pub ref_on_a: bool,
    pub ref_edge: usize,
}

pub fn sat(a: &Obb, b: &Obb) -> Option<SatResult> {
    let test_axes = [
        (a.axes[0], true),
        (a.axes[1], true),
        (b.axes[0], false),
        (b.axes[1], false),
    ];

    let mut min_pen = f32::MAX;
    let mut normal = Vec2::ZERO;
    let mut ref_on_a = true;

    for (axis, is_a) in test_axes {
        let (a_min, a_max) = a.project(axis);
        let (b_min, b_max) = b.project(axis);

        let overlap = a_max.min(b_max) - a_min.max(b_min);
        if overlap <= 0.0 {
            return None;
        }

        if overlap < min_pen {
            min_pen = overlap;
            normal = axis;
            ref_on_a = is_a;
            // Ensure normal points from A to B
            let d = b.center - a.center;
            if normal.dot(d) < 0.0 {
                normal = -normal;
            }
        }
    }

    // The normal is one of the reference box's axes (possibly negated).
    // Map it to the edge whose outward normal matches.
    let ref_box = if ref_on_a { a } else { b };
    let ref_edge = match normal.dot(ref_box.axes[0]) {
        d if d > 0.99 => 1,  // +X edge
        d if d < -0.99 => 3, // -X edge
        _ => match normal.dot(ref_box.axes[1]) {
            d if d > 0.99 => 2,  // +Y edge
            d if d < -0.99 => 0, // -Y edge
            _ => 0, // shouldn't happen for axis-aligned SAT
        },
    };

    Some(SatResult {
        normal,
        penetration: min_pen,
        ref_on_a,
        ref_edge,
    })
}

pub struct ClipPoint {
    pub world: Vec2,
    pub separation: f32,
    pub inc_edge: usize,
    pub inc_vertex: usize,
}

const MAX_CLIP: usize = 2;

pub fn clip(
    a: &Obb,
    b: &Obb,
    sat: &SatResult,
) -> [Option<ClipPoint>; MAX_CLIP] {
    let (ref_box, inc_box) = if sat.ref_on_a {
        (a, b)
    } else {
        (b, a)
    };

    let ref_corners = ref_box.corners();
    let inc_corners = inc_box.corners();

    let ref_edge = sat.ref_edge;
    let ref_start = ref_corners[ref_edge];
    let ref_end = ref_corners[(ref_edge + 1) % 4];

    // Find the incident edge: the edge most anti-parallel to the normal
    let mut min_dot = f32::MAX;
    let mut inc_edge = 0;
    for i in 0..4 {
        let dot = inc_box.edge_normal(i).dot(sat.normal);
        if dot < min_dot {
            min_dot = dot;
            inc_edge = i;
        }
    }

    let inc_start = inc_corners[inc_edge];
    let inc_end = inc_corners[(inc_edge + 1) % 4];

    let edge_dir = (ref_end - ref_start).normalize_or_zero();

    // Clip incident edge against the two side planes of the reference edge
    let clipped = clip_segment(inc_start, inc_end, ref_start, edge_dir);
    let clipped = if clipped[0].is_some() && clipped[1].is_some() {
        clip_segment(clipped[0].unwrap().0, clipped[1].unwrap().0, ref_end, -edge_dir)
    } else {
        [None, None]
    };

    // Keep only points behind the reference plane (penetrating)
    let mut result = [None, None];
    let mut count = 0;
    for (point, vertex_idx) in clipped.iter().flatten() {
        let sep = (*point - ref_start).dot(sat.normal);
        if sep <= 0.0 && count < MAX_CLIP {
            result[count] = Some(ClipPoint {
                world: *point,
                separation: sep,
                inc_edge,
                inc_vertex: *vertex_idx,
            });
            count += 1;
        }
    }

    result
}

fn clip_segment(
    v1: Vec2,
    v2: Vec2,
    plane_point: Vec2,
    plane_normal: Vec2,
) -> [Option<(Vec2, usize)>; MAX_CLIP] {
    let d1 = (v1 - plane_point).dot(plane_normal);
    let d2 = (v2 - plane_point).dot(plane_normal);

    if d1 >= 0.0 && d2 >= 0.0 {
        [Some((v1, 0)), Some((v2, 1))]
    } else if d1 >= 0.0 {
        let t = d1 / (d1 - d2);
        [Some((v1, 0)), Some((v1 + (v2 - v1) * t, 0))]
    } else if d2 >= 0.0 {
        let t = d1 / (d1 - d2);
        [Some((v1 + (v2 - v1) * t, 1)), Some((v2, 1))]
    } else {
        [None, None]
    }
}

pub fn world_to_local(world: Vec2, center: Vec3, rot: Quat) -> Vec3 {
    let inv = rot.inverse();
    let world3 = Vec3::new(world.x, world.y, 0.0) - center;
    inv * world3
}
