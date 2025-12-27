use crate::solver::SolverRandomState;
use godot::global::randi_range;

pub struct GodotRandom {}

impl SolverRandomState for GodotRandom {
    fn next_in_range(&mut self, max: usize) -> usize {
        randi_range(0, (max as i64) - 1) as usize
    }
}
