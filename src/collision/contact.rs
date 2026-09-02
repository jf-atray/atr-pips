use glam::Vec3;

use crate::ecs::PipId;

const MAX_POINTS: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ManifoldPoint {
    pub local_a: Vec3,
    pub local_b: Vec3,
    pub separation: f32,
    pub contact_id: u16,
    pub normal_impulse: f32,
    pub tangent_impulse: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ContactPair {
    pub body_a: PipId,
    pub body_b: PipId,
    pub normal: Vec3,
    pub friction: f32,
    pub restitution: f32,
    pub points: [ManifoldPoint; MAX_POINTS],
    pub point_count: u8,
    pub touched: bool,
}

impl ContactPair {
    pub fn key(&self) -> (PipId, PipId) {
        (self.body_a, self.body_b)
    }

    pub fn active_points(&self) -> &[ManifoldPoint] {
        &self.points[..self.point_count as usize]
    }
}

#[derive(Debug, Default)]
pub struct ContactCache {
    pairs: Vec<ContactPair>,
}

impl ContactCache {
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn pairs(&self) -> &[ContactPair] {
        &self.pairs
    }

    pub fn pairs_mut(&mut self) -> &mut [ContactPair] {
        &mut self.pairs
    }

    pub fn find(&self, key: (PipId, PipId)) -> Result<usize, usize> {
        self.pairs.binary_search_by_key(&key, |p| p.key())
    }

    pub fn begin_frame(&mut self) {
        for pair in &mut self.pairs {
            pair.touched = false;
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut ContactPair {
        &mut self.pairs[index]
    }

    pub fn insert(&mut self, index: usize, pair: ContactPair) -> &mut ContactPair {
        self.pairs.insert(index, pair);
        &mut self.pairs[index]
    }

    pub fn evict_untouched(&mut self) {
        self.pairs.retain(|p| p.touched);
    }
}
