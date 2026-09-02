use std::collections::{HashMap, HashSet};

use glam::Vec3;
use slotmap::Key;

use crate::ecs::PipId;
use crate::spacial::aabb::Aabb;

type CellKey = u64;

fn hash_cell(x: i32, y: i32, z: i32) -> CellKey {
    let x = x as u64;
    let y = y as u64;
    let z = z as u64;
    (x.wrapping_mul(73856093))
        ^ (y.wrapping_mul(19349663))
        ^ (z.wrapping_mul(83492791))
}

#[derive(Debug)]
pub struct SpatialHash {
    pub cell_size: f32,
    cells: HashMap<CellKey, Vec<PipId>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::with_capacity(1024),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, pip: PipId, aabb: &Aabb) {
        let inv = 1.0 / self.cell_size;
        let min_x = (aabb.min.x * inv).floor() as i32;
        let min_y = (aabb.min.y * inv).floor() as i32;
        let min_z = (aabb.min.z * inv).floor() as i32;
        let max_x = (aabb.max.x * inv).floor() as i32;
        let max_y = (aabb.max.y * inv).floor() as i32;
        let max_z = (aabb.max.z * inv).floor() as i32;

        for cx in min_x..=max_x {
            for cy in min_y..=max_y {
                for cz in min_z..=max_z {
                    let key = hash_cell(cx, cy, cz);
                    let bucket = self.cells.entry(key).or_default();
                    if bucket.is_empty() {
                        bucket.reserve(32);
                    }
                    bucket.push(pip);
                }
            }
        }
    }

    pub fn cells(&self) -> &HashMap<CellKey, Vec<PipId>> {
        &self.cells
    }
}

type PairKey = (u64, u64);

fn pair_key(a: PipId, b: PipId) -> PairKey {
    let ai = a.data().as_ffi() as u64;
    let bi = b.data().as_ffi() as u64;
    if ai < bi { (ai, bi) } else { (bi, ai) }
}

#[derive(Debug, Default)]
pub struct CandidatePairs {
    pub pairs: Vec<(PipId, PipId)>,
    seen: HashSet<PairKey>,
}

impl CandidatePairs {
    pub fn begin_frame(&mut self, pip_capacity: usize) {
        self.pairs.clear();
        self.seen.clear();
        if self.pairs.capacity() < pip_capacity * 4 {
            self.pairs.reserve(pip_capacity * 4);
        }
        if self.seen.capacity() < pip_capacity * 4 {
            self.seen.reserve(pip_capacity * 4);
        }
    }

    pub fn try_add(&mut self, a: PipId, b: PipId) {
        let key = pair_key(a, b);
        if self.seen.insert(key) {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            self.pairs.push((lo, hi));
        }
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }
}

impl SpatialHash {
    pub fn cell_extent(&self, aabb: &Aabb) -> Vec3 {
        let inv = 1.0 / self.cell_size;
        Vec3::new(
            (aabb.max.x * inv).floor() as f32 - (aabb.min.x * inv).floor() as f32 + 1.0,
            (aabb.max.y * inv).floor() as f32 - (aabb.min.y * inv).floor() as f32 + 1.0,
            (aabb.max.z * inv).floor() as f32 - (aabb.min.z * inv).floor() as f32 + 1.0,
        )
    }
}
