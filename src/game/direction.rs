use glam::Vec2;

/// The [Direction] enum represents the directions of the game
/// It is used for movement, collisions, and orientating objects
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns all the [Direction]s as an iterator
    pub fn iterator() -> impl Iterator<Item = &'static Direction> {
        static DIRECTIONS: [Direction; 4] = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        DIRECTIONS.iter()
    }

    /// Converts a Direction to the corresponding normalized [vector](Vec2) in game logic
    pub fn to_vec2(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0.0, -1.0),
            Direction::Down => Vec2::new(0.0, 1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Right => Vec2::new(1.0, 0.0),
        }
    }

    /// Transforms an input and output positions into a sensible Direction to go from the first to the latter, made for AI pathfinding
    pub fn get_direction(start: &Vec2, goal: &Vec2) -> Direction {
        let delta = *goal - *start;
        match (delta.x, delta.y) {
            (x, _) if x > 0.0 => Direction::Right,
            (x, _) if x < 0.0 => Direction::Left,
            (_, y) if y > 0.0 => Direction::Down,
            (_, y) if y < 0.0 => Direction::Up,
            (_, _) => Direction::Up,
        }
    }
}
