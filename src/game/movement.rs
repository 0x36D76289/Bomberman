use crate::game::{
    direction::Direction,
    grid::{Tile, center_of, step, tile_of},
};
use glam::Vec2;

const EPSILON: f32 = 1e-4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Walk {
    pub travelled: f32,
    pub facing: Option<Direction>,
    pub bumped: Option<(Tile, Direction)>,
}

fn along(pos: Vec2, direction: Direction) -> f32 {
    match direction {
        Direction::Left | Direction::Right => pos.x,
        Direction::Up | Direction::Down => pos.y,
    }
}

fn across(pos: Vec2, direction: Direction) -> f32 {
    match direction {
        Direction::Left | Direction::Right => pos.y,
        Direction::Up | Direction::Down => pos.x,
    }
}

fn perpendicular(direction: Direction) -> Vec2 {
    match direction {
        Direction::Left | Direction::Right => Vec2::Y,
        Direction::Up | Direction::Down => Vec2::X,
    }
}

fn walk_towards(
    pos: &mut Vec2,
    direction: Direction,
    mut budget: f32,
    blocked: &impl Fn(Tile) -> bool,
) -> (f32, Option<Tile>) {
    let start_budget = budget;
    let forward = direction.to_vec2();
    let side = perpendicular(direction);

    let tile = tile_of(*pos);
    let offset = across(*pos, direction) - across(center_of(tile), direction);
    if offset.abs() > EPSILON {
        let neighbour = (
            tile.0 + (side.x * offset.signum()) as i32,
            tile.1 + (side.y * offset.signum()) as i32,
        );
        let lane = if !blocked(step(tile, direction)) {
            tile
        } else if 1.0 - offset.abs() <= 0.6
            && !blocked(neighbour)
            && !blocked(step(neighbour, direction))
        {
            neighbour
        } else {
            return (0.0, None);
        };
        let gap = across(center_of(lane), direction) - across(*pos, direction);
        let moved = gap.abs().min(budget);
        *pos += side * gap.signum() * moved;
        budget -= moved;
        if (gap.abs() - moved) > EPSILON {
            return (start_budget - budget, None);
        }
        let center = center_of(lane);
        match direction {
            Direction::Left | Direction::Right => pos.y = center.y,
            Direction::Up | Direction::Down => pos.x = center.x,
        }
    }

    while budget > EPSILON {
        let tile = tile_of(*pos);
        let to_center = (along(center_of(tile), direction) - along(*pos, direction))
            * along(forward, direction).signum();
        let moved = if to_center > EPSILON {
            to_center.min(budget)
        } else {
            let ahead = step(tile, direction);
            if blocked(ahead) {
                if to_center.abs() <= EPSILON {
                    *pos = center_of(tile);
                }
                return (start_budget - budget, Some(ahead));
            }
            (1.0 + to_center).min(budget)
        };
        *pos += forward * moved;
        budget -= moved;
    }
    (start_budget - budget, None)
}

pub fn walk(
    pos: &mut Vec2,
    wishes: &[Direction],
    distance: f32,
    blocked: impl Fn(Tile) -> bool,
) -> Walk {
    let mut result = Walk {
        travelled: 0.0,
        facing: wishes.first().copied(),
        bumped: None,
    };
    let mut budget = distance;
    let mut faced = false;
    for direction in wishes {
        let (travelled, bumped) = walk_towards(pos, *direction, budget, &blocked);
        if travelled > EPSILON && !faced {
            result.facing = Some(*direction);
            faced = true;
        }
        if result.bumped.is_none() {
            result.bumped = bumped.map(|tile| (tile, *direction));
        }
        result.travelled += travelled;
        budget -= travelled;
        if budget <= EPSILON {
            break;
        }
    }
    result
}
