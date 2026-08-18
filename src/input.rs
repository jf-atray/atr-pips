use std::collections::HashMap;

use glam::Vec2;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub struct AxisConfig {
    pub positive: Vec<KeyCode>,
    pub negative: Vec<KeyCode>,
    pub positive_mouse: Vec<MouseButton>,
    pub negative_mouse: Vec<MouseButton>,
}

impl AxisConfig {
    pub fn new(positive: Vec<KeyCode>, negative: Vec<KeyCode>) -> Self {
        Self {
            positive,
            negative,
            positive_mouse: Vec::new(),
            negative_mouse: Vec::new(),
        }
    }

    pub fn with_mouse(
        mut self,
        positive_mouse: Vec<MouseButton>,
        negative_mouse: Vec<MouseButton>,
    ) -> Self {
        self.positive_mouse = positive_mouse;
        self.negative_mouse = negative_mouse;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Key {
        code: KeyCode,
        state: ElementState,
        repeat: bool,
    },
    Char(char),
    ImeCommit(String),
    MouseMove(Vec2),
    MouseButton {
        button: MouseButton,
        state: ElementState,
    },
    MouseScroll(Vec2),
}

#[derive(Debug, Clone, Default)]
pub struct Mouse {
    pub pos: Vec2,
    pub delta: Vec2,
    pub scroll: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct Text {
    pub committed: String,
}

#[derive(Debug, Clone, Default)]
pub struct VirtualAxes {
    pub configs: HashMap<&'static str, AxisConfig>,
    states: HashMap<&'static str, (i32, Option<i32>)>,
}

impl VirtualAxes {
    pub fn value(&self, name: &str) -> f32 {
        self.states
            .get(name)
            .map(|(c, _)| c.signum() as f32)
            .unwrap_or(0.0)
    }

    pub fn delta(&self, name: &str) -> Option<i32> {
        self.states.get(name).and_then(|(_, d)| *d)
    }

    fn update(&mut self, key: &KeyCode, down: bool) {
        let delta = if down { 1 } else { -1 };
        for (name, config) in &self.configs {
            let in_pos = config.positive.contains(key);
            let in_neg = config.negative.contains(key);
            if !in_pos && !in_neg {
                continue;
            }
            let (count, _frame) = self.states.entry(name.clone()).or_insert((0, None));
            if in_pos {
                *count += delta;
            }
            if in_neg {
                *count -= delta;
            }
            *_frame = Some(*count);
        }
    }

    fn update_mouse(&mut self, button: &MouseButton, down: bool) {
        let delta = if down { 1 } else { -1 };
        for (name, config) in &self.configs {
            let in_pos = config.positive_mouse.contains(button);
            let in_neg = config.negative_mouse.contains(button);
            if !in_pos && !in_neg {
                continue;
            }
            let (count, _frame) = self.states.entry(name.clone()).or_insert((0, None));
            if in_pos {
                *count += delta;
            }
            if in_neg {
                *count -= delta;
            }
            *_frame = Some(*count);
        }
    }

    fn clear_deltas(&mut self) {
        for (_, d) in self.states.values_mut() {
            *d = None;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Input {
    pub mouse: Mouse,
    pub text: Text,
    pub axes: VirtualAxes,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_axis(&mut self, name: &'static str, config: AxisConfig) {
        self.axes.configs.insert(name, config);
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::Key {
                code,
                state,
                repeat,
            } => {
                if !repeat {
                    let down = state == ElementState::Pressed;
                    self.axes.update(&code, down);
                }
            }
            InputEvent::Char(c) => self.text.committed.push(c),
            InputEvent::ImeCommit(s) => self.text.committed.push_str(&s),
            InputEvent::MouseMove(pos) => {
                self.mouse.delta += pos - self.mouse.pos;
                self.mouse.pos = pos;
            }
            InputEvent::MouseButton { button, state } => {
                let down = state == ElementState::Pressed;
                self.axes.update_mouse(&button, down);
            }
            InputEvent::MouseScroll(delta) => self.mouse.scroll += delta,
        }
    }

    pub fn end_frame(&mut self) {
        self.text.committed.clear();
        self.mouse.delta = Vec2::ZERO;
        self.mouse.scroll = Vec2::ZERO;
        self.axes.clear_deltas();
    }
}
