use glam::Vec2;

// LOIC: Very low, should be used only for position
const EPSILON: f32 = 1e-1;

pub trait ApproxEq {
    fn approx_eq(&self, other: &Self) -> bool;
}

pub trait Grid {
    fn grid(&self) -> Self;
}

/// Compare the value of vec1 to vec2
impl ApproxEq for Vec2 {
    fn approx_eq(&self, other: &Vec2) -> bool {
        (self.x - other.x).abs() < EPSILON && (self.y - other.y).abs() < EPSILON
    }
}

/// "Grid" the values inside the vector to put them in the middle
impl Grid for Vec2 {
    fn grid(&self) -> Self {
        Vec2::new(self.x.floor() + 0.5, self.y.floor() + 0.5)
    }
}
