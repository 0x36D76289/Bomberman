use super::map::Map;
use crate::game::resources::Resources;

/// The various presets of map creation
pub enum MapType {
    /// The Corners preset is meant for 4 players
    Corners,
    /// The Arena preset is meant for 10 players (from the Genesis classic)
    Arena,
    /// The Teams preset is meant for 8 players in a 2v2v2v2 configuration
    Teams,
    /// The Random map type enables an arbitrary amount of players
    Random,
}

/// The [MapSettings] control all the instructions in the [Map new constructor](Map::new())
pub struct MapSettings {
    pub width: u8,
    pub height: u8,
    pub cheesiness: u8,
    pub spawns: u8,
    pub spawn_size: u8,
    pub safe_range: u8,
    pub map_type: MapType,
    pub walls: bool,
    pub attempts: u8,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self::corners()
    }
}

impl MapSettings {
    /// The corners map preset MapSettings constructor
    pub fn corners() -> Self {
        MapSettings {
            width: 15,
            height: 15,
            cheesiness: 5,
            spawns: 4,
            spawn_size: 1,
            safe_range: 3,
            map_type: MapType::Corners,
            walls: true,
            attempts: 100,
        }
    }

    /// The arena map preset MapSettings constructor
    pub fn arena() -> Self {
        MapSettings {
            width: 37,
            height: 21,
            cheesiness: 7,
            spawns: 10,
            spawn_size: 1,
            safe_range: 3,
            map_type: MapType::Arena,
            walls: true,
            attempts: 100,
        }
    }

    /// The teams map preset MapSettings constructor
    pub fn teams() -> Self {
        MapSettings {
            width: 21,
            height: 21,
            cheesiness: 2,
            spawns: 8,
            spawn_size: 0,
            safe_range: 3,
            map_type: MapType::Teams,
            walls: true,
            attempts: 100,
        }
    }

    /// The cheese map preset MapSettings constructor
    pub fn default_cheese() -> Self {
        MapSettings {
            width: 15,
            height: 15,
            cheesiness: 5,
            spawns: 4,
            spawn_size: 1,
            safe_range: 3,
            map_type: MapType::Random,
            walls: true,
            attempts: 100,
        }
    }

    /// creates a map from the current map settings
    pub fn new_map(settings: Self, resources: &Resources) -> Option<Map> {
        Map::new(settings, resources)
    }
}
