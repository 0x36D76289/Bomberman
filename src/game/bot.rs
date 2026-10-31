use crate::{
    game::{
        bomb::BOMB_TIMER_DEFAULT,
        direction::Direction,
        grid::{Cell, Paths, Tile, TileGrid, center_of, manhattan, step, tile_of},
        player::Player,
        powerup::{PowerUp, PowerUpType},
    },
    input::input::Input,
};
use glam::Vec2;
use rand::{Rng, RngExt, seq::IndexedRandom};

const SEARCH_DEPTH: u32 = 40;
const FLEE_DEPTH: u32 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BotLevel {
    Easy,
    #[default]
    Normal,
    Hard,
}

struct Skill {
    reaction: f32,
    margin: f32,
    aggression: f32,
    greed: f32,
    kicks: bool,
    traps: bool,
    careful: bool,
    alertness: f32,
}

impl BotLevel {
    pub const ALL: [BotLevel; 3] = [BotLevel::Easy, BotLevel::Normal, BotLevel::Hard];

    pub fn name(self) -> &'static str {
        match self {
            BotLevel::Easy => "Facile",
            BotLevel::Normal => "Normal",
            BotLevel::Hard => "Difficile",
        }
    }

    pub fn stepped(self, steps: i32) -> Self {
        Self::ALL[(Self::ALL.iter().position(|l| *l == self).unwrap_or(1) as i32 + steps)
            .clamp(0, Self::ALL.len() as i32 - 1) as usize]
    }

    fn skill(self) -> Skill {
        match self {
            BotLevel::Easy => Skill {
                reaction: 0.45,
                margin: 0.35,
                aggression: 0.35,
                greed: 0.5,
                kicks: false,
                traps: false,
                careful: false,
                alertness: 0.35,
            },
            BotLevel::Normal => Skill {
                reaction: 0.2,
                margin: 0.25,
                aggression: 0.8,
                greed: 1.0,
                kicks: true,
                traps: false,
                careful: true,
                alertness: 0.12,
            },
            BotLevel::Hard => Skill {
                reaction: 0.05,
                margin: 0.15,
                aggression: 1.0,
                greed: 1.5,
                kicks: true,
                traps: true,
                careful: true,
                alertness: 0.0,
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Bot {
    level: BotLevel,
    goal: Option<Tile>,
    cooldown: f32,
    checked: Option<Tile>,
    alarm: f32,
    previous: Input,
}

pub struct BotView<'a> {
    pub grid: &'a TileGrid,
    pub blast: &'a [f32],
    pub players: &'a [Player],
    pub power_ups: &'a [PowerUp],
}

fn blast_at(grid: &TileGrid, blast: &[f32], tile: Tile) -> f32 {
    grid.index(tile).map_or(f32::INFINITY, |i| blast[i])
}

fn flee_paths(grid: &TileGrid, blast: &[f32], start: Tile, speed: f32, margin: f32) -> Paths {
    grid.bfs(start, FLEE_DEPTH, |tile, depth| {
        let time = blast_at(grid, blast, tile);
        time.is_infinite() || (depth as f32 + 0.5) / speed + margin < time
    })
}

fn escape(grid: &TileGrid, blast: &[f32], start: Tile, speed: f32, margin: f32) -> Option<Tile> {
    let paths = flee_paths(grid, blast, start, speed, margin);
    paths.first_step(
        *paths
            .order
            .iter()
            .find(|tile| blast_at(grid, blast, **tile).is_infinite())?,
    )
}

fn escape_routes(
    grid: &TileGrid,
    blast: &[f32],
    start: Tile,
    speed: f32,
    margin: f32,
) -> (usize, usize) {
    let paths = flee_paths(grid, blast, start, speed, margin);
    let mut first_steps = Vec::new();
    let mut safe = 0;
    for tile in &paths.order {
        if blast_at(grid, blast, *tile).is_infinite() {
            safe += 1;
            if let Some(first) = paths.first_step(*tile)
                && !first_steps.contains(&first)
            {
                first_steps.push(first);
            }
        }
    }
    (safe, first_steps.len())
}

impl BotView<'_> {
    fn opponents<'a>(&'a self, me: &'a Player) -> impl Iterator<Item = &'a Player> + 'a {
        self.players
            .iter()
            .filter(move |p| p.alive && p.id != me.id && (p.team.is_none() || p.team != me.team))
    }

    fn occupied(&self, me: &Player, tile: Tile) -> bool {
        self.players
            .iter()
            .any(|p| p.alive && p.id != me.id && p.footprint().contains(&tile))
    }

    fn blast(&self, tile: Tile) -> f32 {
        blast_at(self.grid, self.blast, tile)
    }

    fn safe(&self, tile: Tile) -> bool {
        self.blast(tile).is_infinite()
    }

    fn power_up_at(&self, tile: Tile) -> bool {
        self.power_ups
            .iter()
            .any(|p| p.power_up_type != PowerUpType::Skull && p.tile == tile)
    }

    fn bomb_value(&self, me: &Player, tile: Tile) -> (u32, u32) {
        let mut blocks = 0;
        let mut hits = 0;
        self.grid.for_each_blast_tile(tile, me.powers.fire, |t| {
            if self.grid.cell(t) == Cell::Soft && self.safe(t) {
                blocks += 1;
            }
            hits += self
                .opponents(me)
                .filter(|p| tile_of(p.position) == t)
                .count() as u32;
        });
        (blocks, hits)
    }

    fn can_escape_own_bomb(&self, me: &Player, tile: Tile, margin: f32) -> bool {
        let grid = self
            .grid
            .with_bomb(tile, me.powers.fire, BOMB_TIMER_DEFAULT);
        escape(&grid, &grid.blast_times(), tile, me.speed(), margin).is_some()
    }

    fn traps_someone(&self, me: &Player, tile: Tile) -> bool {
        let grid = self
            .grid
            .with_bomb(tile, me.powers.fire, BOMB_TIMER_DEFAULT);
        let blast = grid.blast_times();
        self.opponents(me).any(|p| {
            let at = tile_of(p.position);
            manhattan(at, tile) <= 6
                && blast_at(&grid, &blast, at).is_finite()
                && escape(&grid, &blast, at, p.speed(), 0.0).is_none()
        })
    }

    fn kick_direction(&self, me: &Player) -> Option<Direction> {
        Direction::iterator().copied().find(|direction| {
            let bomb_tile = step(tile_of(me.position), *direction);
            if self.grid.bomb_at(bomb_tile).is_none() {
                return false;
            }
            let mut tile = step(bomb_tile, *direction);
            for _ in 0..7 {
                if !self.grid.walkable(tile) {
                    return false;
                }
                if self.opponents(me).any(|p| tile_of(p.position) == tile) {
                    return true;
                }
                tile = step(tile, *direction);
            }
            false
        })
    }
}

impl Bot {
    pub fn new(level: BotLevel) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    pub fn level(&self) -> BotLevel {
        self.level
    }

    fn choose_goal(&self, me: &Player, view: &BotView, rng: &mut impl Rng) -> Option<Tile> {
        let skill = self.level.skill();
        let paths = view
            .grid
            .bfs(tile_of(me.position), SEARCH_DEPTH, |tile, _| {
                view.safe(tile)
            });

        let mut candidates: Vec<(f32, Tile, bool)> = Vec::new();
        for tile in paths.order.iter().take(200) {
            let mut value = 0.0;
            if view.power_up_at(*tile) {
                value += 8.0 * skill.greed;
            }
            let mut bombing = false;
            if me.bombs_remaining > 0 {
                let (blocks, hits) = view.bomb_value(me, *tile);
                if blocks + hits > 0 {
                    value += blocks as f32 * 2.0 + hits as f32 * 5.0 * skill.aggression;
                    bombing = true;
                }
            }
            if value > 0.0 {
                candidates.push((
                    value * rng.random_range(0.9..1.1)
                        / (paths.distance(*tile).unwrap_or(0) as f32 + 2.0),
                    *tile,
                    bombing,
                ));
            }
        }
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
        if let Some((_, tile, _)) = candidates.iter().take(6).find(|(_, tile, bombing)| {
            !bombing || view.can_escape_own_bomb(me, *tile, skill.margin)
        }) {
            return Some(*tile);
        }

        let opponent = view
            .opponents(me)
            .map(|p| tile_of(p.position))
            .filter(|t| paths.distance(*t).is_some())
            .min_by_key(|t| paths.distance(*t));
        if self.level != BotLevel::Easy && opponent.is_some() {
            return opponent;
        }
        paths.order.choose(rng).copied()
    }

    pub fn think(&mut self, me: &Player, view: &BotView, delta: f32) -> Input {
        if !me.alive {
            self.previous = Input::default();
            return self.previous;
        }
        let skill = self.level.skill();
        let mut rng = rand::rng();
        let here = tile_of(me.position);
        let mut place_bomb = false;
        let mut kick = None;

        let threatened = !view.safe(here);
        self.alarm = if threatened { self.alarm + delta } else { 0.0 };
        self.previous = Input::from_motion(
            match (
                if threatened && self.alarm < skill.alertness {
                    None
                } else if threatened {
                    self.goal = None;
                    escape(view.grid, view.blast, here, me.speed(), skill.margin).or_else(|| {
                        let paths = view
                            .grid
                            .bfs(here, FLEE_DEPTH, |tile, _| view.blast(tile) > 0.0);
                        paths.first_step(paths.order.iter().fold(here, |best, tile| {
                            if view.blast(*tile) > view.blast(best) + 0.05 {
                                *tile
                            } else {
                                best
                            }
                        }))
                    })
                } else {
                    let centered = (center_of(here) - me.position).abs().max_element() < 0.12;
                    let at_goal = centered && self.goal == Some(here);
                    self.cooldown -= delta;

                    let mut escape_step = None;
                    if centered && (self.checked != Some(here) || at_goal) {
                        self.checked = Some(here);
                        if me.bombs_remaining > 0 && view.grid.bomb_at(here).is_none() {
                            let (blocks, hits) = view.bomb_value(me, here);
                            if (hits > 0 && rng.random::<f32>() < skill.aggression)
                                || (at_goal && blocks > 0)
                                || (skill.traps && view.traps_someone(me, here))
                            {
                                let grid =
                                    view.grid
                                        .with_bomb(here, me.powers.fire, BOMB_TIMER_DEFAULT);
                                let blast = grid.blast_times();
                                escape_step = escape(&grid, &blast, here, me.speed(), skill.margin);
                                let (safe, routes) =
                                    escape_routes(&grid, &blast, here, me.speed(), skill.margin);
                                place_bomb = escape_step.is_some()
                                    && (!skill.careful
                                        || !view
                                            .opponents(me)
                                            .any(|p| manhattan(tile_of(p.position), here) <= 3)
                                        || routes >= 2
                                        || safe >= 6);
                            }
                        }
                        if skill.kicks && me.powers.kick && !place_bomb {
                            kick = view.kick_direction(me);
                        }
                    }

                    if place_bomb
                        || at_goal
                        || self
                            .goal
                            .is_none_or(|g| !view.grid.walkable(g) || !view.safe(g))
                        || self.cooldown <= 0.0
                    {
                        self.cooldown = skill.reaction + 1.0;
                        self.goal = None;
                        if !place_bomb {
                            self.goal = self.choose_goal(me, view, &mut rng);
                        }
                    }
                    escape_step.or_else(|| {
                        self.goal.and_then(|goal| {
                            let route = |dodge: bool| {
                                view.grid
                                    .bfs(here, SEARCH_DEPTH, |tile, _| {
                                        view.safe(tile)
                                            && (!dodge || tile == goal || !view.occupied(me, tile))
                                    })
                                    .first_step(goal)
                            };
                            route(true).or_else(|| route(false))
                        })
                    })
                },
                kick,
            ) {
                (_, Some(direction)) => direction.to_vec2(),
                (Some(next), None) => Self::motion_to(me.position, here, next),
                (None, None) => Vec2::ZERO,
            },
            place_bomb,
            &self.previous,
        );
        self.previous
    }

    fn motion_to(pos: Vec2, here: Tile, next: Tile) -> Vec2 {
        if next != here {
            return Vec2::new((next.0 - here.0) as f32, (next.1 - here.1) as f32);
        }
        let gap = center_of(here) - pos;
        if gap.abs().max_element() < 0.05 {
            Vec2::ZERO
        } else if gap.x.abs() > gap.y.abs() {
            Vec2::new(gap.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, gap.y.signum())
        }
    }
}
