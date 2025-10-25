/// The [Bomb](bomb::Bomb) object and logic
pub mod bomb;
/// The [Camera](camera::Camera) object
pub mod camera;
/// The [Collision](collision::Collision) trait
pub mod collision;
/// The [Direction](direction::Direction) enum and its logic
pub mod direction;
/// The NPC characters of the singleplayer campaign
pub mod enemy;
/// The [GameSettings](game_settings::GameSettings) structure is used to create new [GameStates](game_state::GameState)
pub mod game_settings;
/// The main game logic, orchestrating everything together
pub mod game_state;
/// The map module contains all the data relating to the [Map](map::map::Map) structure's
/// components and creation
pub mod map;
/// The [Player](player::Player) represents a human player or a bot in multiplayer games
pub mod player;
/// The [PowerUp](powerup::PowerUp) are bonuses the [Player]s can pick up
pub mod powerup;
/// The list of 3d models and textures used by the game
pub mod resources;
