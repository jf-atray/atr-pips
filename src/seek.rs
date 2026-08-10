use glam::{Vec2, Vec3};

/// Generic goal-seeking component with snap-to-goal behavior.
#[derive(Debug, Clone, Default, PartialEq)]
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

    pub fn into_option(self) -> Seek<Option<T>> {
        Seek::with_params(Some(self.goal), self.speed, self.deadzone)
    }
}

/// Types that can be interpolated toward a goal.
pub trait Seekable: Copy {
    fn diff(self, goal: Self) -> Self;
    fn length(self) -> f32;
    fn normalize(self) -> Self;
    fn apply(self, delta: Self) -> Self;
    fn mul_scalar(self, scalar: f32) -> Self;
}

impl Seekable for Vec3 {
    fn diff(self, goal: Self) -> Self {
        goal - self
    }

    fn length(self) -> f32 {
        self.length()
    }

    fn normalize(self) -> Self {
        self.normalize()
    }

    fn apply(self, delta: Self) -> Self {
        self + delta
    }

    fn mul_scalar(self, scalar: f32) -> Self {
        self * scalar
    }
}

impl Seekable for Vec2 {
    fn diff(self, goal: Self) -> Self {
        goal - self
    }

    fn length(self) -> f32 {
        self.length()
    }

    fn normalize(self) -> Self {
        self.normalize()
    }

    fn apply(self, delta: Self) -> Self {
        self + delta
    }

    fn mul_scalar(self, scalar: f32) -> Self {
        self * scalar
    }
}

impl Seekable for f32 {
    fn diff(self, goal: Self) -> Self {
        goal - self
    }

    fn length(self) -> f32 {
        self.abs()
    }

    fn normalize(self) -> Self {
        if self == 0.0 { 0.0 } else { self.signum() }
    }

    fn apply(self, delta: Self) -> Self {
        self + delta
    }

    fn mul_scalar(self, scalar: f32) -> Self {
        self * scalar
    }
}

impl<T: Seekable> Seekable for Option<T> {
    fn diff(self, goal: Self) -> Self {
        match (self, goal) {
            (Some(curr), Some(g)) => Some(curr.diff(g)),
            _ => None,
        }
    }

    fn length(self) -> f32 {
        self.map(|t| t.length()).unwrap_or(0.0)
    }

    fn normalize(self) -> Self {
        self.map(|t| t.normalize())
    }

    fn apply(self, delta: Self) -> Self {
        match (self, delta) {
            (Some(curr), Some(d)) => Some(curr.apply(d)),
            _ => None,
        }
    }

    fn mul_scalar(self, scalar: f32) -> Self {
        self.map(|t| t.mul_scalar(scalar))
    }
}

pub fn solve_seek_core<T: Seekable>(current: &mut T, goal: T, speed: f32, deadzone: f32, dt: f32) {
    let dif = current.diff(goal);
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

    *current = current.apply(normal.mul_scalar(rate));
}

pub fn solve_seek<T: Seekable>(current: &mut T, seek: &Seek<T>, dt: f32) {
    solve_seek_core(current, seek.goal, seek.speed, seek.deadzone, dt);
}

pub fn solve_seek_option<T: Seekable>(current: &mut T, seek: &Seek<Option<T>>, dt: f32) {
    let Some(goal) = seek.goal else {
        return;
    };
    solve_seek_core(current, goal, seek.speed, seek.deadzone, dt);
}
