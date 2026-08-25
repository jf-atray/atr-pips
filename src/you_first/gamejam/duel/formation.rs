#[derive(Debug, Clone, Copy)]
pub enum Formation {
    SemiCircle { radius: f32 },
    Arc { radius: f32, start: f32, end: f32 },
}

impl Formation {
    pub fn positions(&self, count: usize) -> Vec<(f32, f32)> {
        let (radius, start, end) = match *self {
            Formation::SemiCircle { radius } => (radius, 0.0, std::f32::consts::PI),
            Formation::Arc { radius, start, end } => (radius, start, end),
        };

        if count == 0 {
            return Vec::new();
        }

        let mid = (start + end) * 0.5;
        let (start, end) = if count == 1 {
            (mid, mid)
        } else {
            (start, end)
        };

        (0..count)
            .map(|i| {
                let t = if count > 1 {
                    i as f32 / (count - 1) as f32
                } else {
                    0.0
                };
                let angle = start + (end - start) * t;
                let lateral = radius * angle.cos();
                let d = radius * angle.sin() * 0.5;
                (lateral, d)
            })
            .collect()
    }
}
