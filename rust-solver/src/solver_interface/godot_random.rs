use crate::solver::SolverRandomState;
use godot::global::randi_range;

pub struct GodotRandom {}

impl SolverRandomState for GodotRandom {
    fn next_in_range(&mut self, max: usize) -> usize {
        let max_as_i64 = i64::try_from(max).expect("usize too large to represent as i64.");
        usize::try_from(randi_range(0, max_as_i64 - 1))
            .expect("random value(i64) return is out of return(usize) range")
    }
}
