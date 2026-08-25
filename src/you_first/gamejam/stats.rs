#[derive(Debug, Clone, Default)]
pub struct GameStats {
    pub start: u32,
    pub complete: u32,
    pub challenge: u32,
    pub death: u32,
    pub kills: u32,
    pub highscore: u32,
    pub times_played: u32,
    pub completed_levels: u32,
    pub items_collected: u32,
    pub playtime: u32,
}

impl GameStats {
    pub fn flush(&self) {}
}
