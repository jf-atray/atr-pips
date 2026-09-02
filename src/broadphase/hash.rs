use std::collections::HashMap;

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
            cells: HashMap::new(),
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
                    self.cells.entry(key).or_default().push(pip);
                }
            }
        }
    }

    pub fn cells(&self) -> &HashMap<CellKey, Vec<PipId>> {
        &self.cells
    }
}

#[derive(Debug, Default)]
pub struct CandidatePairs {
    pub pairs: Vec<(PipId, PipId)>,
    frame_stamp: Vec<u32>,
    frame: u32,
}

impl CandidatePairs {
    pub fn begin_frame(&mut self, pip_capacity: usize) {
        self.pairs.clear();
        self.frame = self.frame.wrapping_add(1);
        if self.frame_stamp.len() < pip_capacity {
            self.frame_stamp.resize(pip_capacity, 0);
        }
    }

    pub fn try_add(&mut self, a: PipId, b: PipId) {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let idx = hi.data().as_ffi() as usize;
        if idx < self.frame_stamp.len() && self.frame_stamp[idx] == self.frame {
            return;
        }
        if idx < self.frame_stamp.len() {
            self.frame_stamp[idx] = self.frame;
        }
        self.pairs.push((lo, hi));
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
