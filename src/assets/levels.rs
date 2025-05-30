use bevy::prelude::*;

mod levels {
    pub struct Level {
        pub name: String,
        pub enemy_count: u32,
        pub player_start_position: (f32, f32),
    }

    impl Level {
        pub fn new(name: &str, enemy_count: u32, player_start_position: (f32, f32)) -> Self {
            Level {
                name: name.to_string(),
                enemy_count,
                player_start_position,
            }
        }
    }

    pub fn load_levels() -> Vec<Level> {
        vec![
            Level::new("Level 1", 5, (0.0, 0.0)),
            Level::new("Level 2", 10, (1.0, 1.0)),
            Level::new("Level 3", 15, (2.0, 2.0)),
        ]
    }
}
