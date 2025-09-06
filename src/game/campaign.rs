use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct EnemySpawn {
    pub start: Vec2,
    pub end: Vec2,
}

pub struct Level {
    pub map_layout: &'static str,
    pub enemies: &'static [EnemySpawn],
}

// X=Unbreakable, #=Breakable, .=Empty, S=Player Spawn, E=Enemy patrol point (treated as empty)
pub const LEVELS: [Level; 4] = [
    // LEVEL 1
    Level {
        map_layout: r"
XXXXXXXXXXXXX
X S . . . . X
X.#.#.#.#.#.X
X . . E . . X
X.#.#.#.#.#.X
X . . . . . X
X.#.#.#.#.#.X
XXXXXXXXXXXXX
",
        enemies: &[EnemySpawn {
            start: Vec2::new(4.5, 3.5),
            end: Vec2::new(8.5, 3.5),
        }],
    },
    // LEVEL 2
    Level {
        map_layout: r"
XXXXXXXXXXXXXXX
X S . # . # . X
X .#.#.#.#.#. X
X # . E . . # X
X .#.#.#.#.#. X
X # . . E . # X
X .#.#.#.#.#. X
X . # . . # . X
XXXXXXXXXXXXXXX
",
        enemies: &[
            EnemySpawn {
                start: Vec2::new(4.5, 3.5),
                end: Vec2::new(10.5, 3.5),
            },
            EnemySpawn {
                start: Vec2::new(10.5, 5.5),
                end: Vec2::new(4.5, 5.5),
            },
        ],
    },
    // LEVEL 3
    Level {
        map_layout: r"
XXXXXXXXXXXXX
X S . E . . X
X.#.#.#.#.#.X
X . E . E . X
X.#.#.#.#.#.X
X . . E . . X
X.#.#.#.#.#.X
XXXXXXXXXXXXX
",
        enemies: &[
            EnemySpawn {
                start: Vec2::new(4.5, 1.5),
                end: Vec2::new(8.5, 1.5),
            },
            EnemySpawn {
                start: Vec2::new(2.5, 3.5),
                end: Vec2::new(6.5, 3.5),
            },
            EnemySpawn {
                start: Vec2::new(6.5, 3.5),
                end: Vec2::new(10.5, 3.5),
            },
            EnemySpawn {
                start: Vec2::new(8.5, 5.5),
                end: Vec2::new(4.5, 5.5),
            },
        ],
    },
    // LEVEL 4
    Level {
        map_layout: r"
XXXXXXXXXXXXX
X S # E # E X
X .#.#.#.#. X
X E . . . E X
X .#.#.#.#. X
X E # . # E X
X .#.#.#.#. X
X . # . # . X
XXXXXXXXXXXXX
",
        enemies: &[
            EnemySpawn {
                start: Vec2::new(4.5, 1.5),
                end: Vec2::new(8.5, 1.5),
            },
            EnemySpawn {
                start: Vec2::new(1.5, 3.5),
                end: Vec2::new(10.5, 3.5),
            },
            EnemySpawn {
                start: Vec2::new(1.5, 5.5),
                end: Vec2::new(6.5, 5.5),
            },
            EnemySpawn {
                start: Vec2::new(10.5, 5.5),
                end: Vec2::new(6.5, 5.5),
            },
        ],
    },
];
