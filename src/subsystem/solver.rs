use crate::game::catalog::{get_best_hunt_for_level, HuntTier};

#[derive(Default)]
pub struct RouteSolver;

impl RouteSolver {
    pub fn select_best_hunt(&self, level: u32) -> &'static HuntTier {
        get_best_hunt_for_level(level)
    }
}
