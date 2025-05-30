use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub damage: f32,
    pub material: Handle<StandardMaterial>,
}

#[derive(Debug, Clone, Component)]
pub struct Bullet;
