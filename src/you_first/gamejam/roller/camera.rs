use glam::{Vec2, Vec3};

use crate::spacial::camera::Camera;

pub trait Norm: Copy {
    fn length(self) -> f32;
    fn normalize(self) -> Self;
}

impl Norm for f32 {
    fn length(self) -> f32 {
        self.abs()
    }

    fn normalize(self) -> Self {
        if self == 0.0 { 0.0 } else { self.signum() }
    }
}

impl Norm for Vec2 {
    fn length(self) -> f32 {
        self.length()
    }

    fn normalize(self) -> Self {
        self.normalize_or(Vec2::ZERO)
    }
}

impl Norm for Vec3 {
    fn length(self) -> f32 {
        self.length()
    }

    fn normalize(self) -> Self {
        self.normalize_or(Vec3::ZERO)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Seek<T> {
    pub goal: T,
    pub speed: f32,
    pub deadzone: f32,
}

impl<T> Seek<T> {
    pub fn new(goal: T) -> Self {
        Self {
            goal,
            speed: 0.0,
            deadzone: 0.0,
        }
    }

    pub fn with_speed(goal: T, speed: f32) -> Self {
        Self {
            goal,
            speed,
            deadzone: 0.0,
        }
    }

    pub fn with_params(goal: T, speed: f32, deadzone: f32) -> Self {
        Self {
            goal,
            speed,
            deadzone,
        }
    }
}

impl<T> Seek<T> {
    pub fn into_option(self) -> Seek<Option<T>> {
        Seek::with_params(Some(self.goal), self.speed, self.deadzone)
    }
}

pub fn solve_seek_core<T>(current: &mut T, goal: T, speed: f32, deadzone: f32, dt: f32)
where
    T: Norm
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Mul<f32, Output = T>,
{
    let dif = goal - *current;
    let length = dif.length();

    if length < deadzone {
        return;
    }

    if !speed.is_normal() {
        *current = goal;
        return;
    }

    let normal = dif.normalize();
    let rate = speed * dt;

    if length < rate {
        *current = goal;
        return;
    }

    *current = *current + normal * rate;
}

pub fn solve_seek<T>(current: &mut T, seek: &Seek<T>, dt: f32)
where
    T: Copy
        + Norm
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Mul<f32, Output = T>,
{
    solve_seek_core(current, seek.goal, seek.speed, seek.deadzone, dt);
}

pub fn solve_seek_option<T>(current: &mut T, seek: &Seek<Option<T>>, dt: f32)
where
    T: Copy
        + Norm
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Mul<f32, Output = T>,
{
    let Some(goal) = seek.goal else {
        return;
    };
    solve_seek_core(current, goal, seek.speed, seek.deadzone, dt);
}

#[derive(Clone, Debug)]
pub struct CameraDirector {
    pub pan: Seek<Vec2>,
    pub zoom: Seek<f32>,
}

impl CameraDirector {
    pub fn new(initial_pos: Vec2, initial_zoom: f32, pan_speed: f32, zoom_speed: f32) -> Self {
        Self {
            pan: Seek::with_speed(initial_pos, pan_speed),
            zoom: Seek::with_speed(initial_zoom, zoom_speed),
        }
    }

    pub fn set_pan_target(&mut self, target: Vec2) {
        self.pan.goal = target;
    }

    pub fn set_zoom_target(&mut self, target: f32) {
        self.zoom.goal = target;
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        solve_seek(&mut camera.pos, &self.pan, dt);
        solve_seek(&mut camera.zoom, &self.zoom, dt);
    }
}
