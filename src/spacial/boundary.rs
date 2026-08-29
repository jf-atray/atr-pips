use glam::Vec3;

#[derive(Clone, PartialEq, Debug)]
pub struct Boundary {
    pub min: Vec3,
    pub max: Vec3,
    pub restitution: f32,
}

impl Boundary {
    pub fn unbounded() -> Self {
        Self {
            min: Vec3::splat(f32::NEG_INFINITY),
            max: Vec3::splat(f32::INFINITY),
            restitution: 1.0,
        }
    }

    pub fn reflect(&self, pos: &mut Vec3, vel: &mut Vec3) {
        if pos.x < self.min.x {
            pos.x = self.min.x;
            vel.x *= -self.restitution;
        } else if pos.x > self.max.x {
            pos.x = self.max.x;
            vel.x *= -self.restitution;
        }

        if pos.y < self.min.y {
            pos.y = self.min.y;
            vel.y *= -self.restitution;
        } else if pos.y > self.max.y {
            pos.y = self.max.y;
            vel.y *= -self.restitution;
        }

        if pos.z < self.min.z {
            pos.z = self.min.z;
            vel.z *= -self.restitution;
        } else if pos.z > self.max.z {
            pos.z = self.max.z;
            vel.z *= -self.restitution;
        }
    }
}

impl Default for Boundary {
    fn default() -> Self {
        Self::unbounded()
    }
}
