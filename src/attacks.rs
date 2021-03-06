use std::f64::consts;

use bevy::{
    math,
    prelude::{Commands, Sprite, SpriteBundle, Transform},
};
use bevy::{
    math::{Vec2, Vec3},
    prelude::Handle,
    sprite::ColorMaterial,
};

use crate::Collider;
use crate::Fireball;

// pub fn default(
//     commands: &mut Commands,
//     origin: &Vec3,
//     target: &Vec3,
//     fire_sp: &Handle<ColorMaterial>,
// ) {
//     commands
//         .spawn(SpriteBundle {
//             material: fire_sp.clone(),
//             transform: Transform::from_translation(*origin),
//             sprite: Sprite::new(Vec2::new(32.0, 32.0)),
//             ..Default::default()
//         })
//         .with(Fireball {
//             origin: *origin,
//             target: *target,
//         })
//         .with(Collider::Projectile);
// }

pub struct Split;
impl Attack for Split {
    fn attack(
        self: &Self,
        commands: &mut Commands,
        origin: &Vec3,
        target: &Vec3,
        fire_sp: &Handle<ColorMaterial>,
    ) {
        let up: Vec3 = Vec3::unit_y() * 100.0;
        let raw_target = *target - *origin;
        let angle = up.angle_between(raw_target) + 10f32.to_radians() * std::f32::consts::PI;

        let diff1 = Vec3::new(angle.cos(), angle.sin(), 0.0) * 100.0;

        let angle = up.angle_between(raw_target) - 10f32.to_radians() * std::f32::consts::PI;
        let diff2 = Vec3::new(angle.cos(), angle.sin(), 0.0) * 100.0;
        println!("diff: {}", diff1);

        let target_1 = Vec3::new(target.x + diff1.x, target.y + diff1.y, 0.0);
        let target_2 = Vec3::new(target.x - diff2.x, target.y - diff2.y, 0.0);

        commands
            .spawn(SpriteBundle {
                material: fire_sp.clone(),
                transform: Transform::from_translation(*origin),
                sprite: Sprite::new(Vec2::new(32.0, 32.0)),
                ..Default::default()
            })
            .with(Fireball {
                origin: *origin,
                target: target_1,
            })
            .with(Collider::Projectile);

        commands
            .spawn(SpriteBundle {
                material: fire_sp.clone(),
                transform: Transform::from_translation(*origin),
                sprite: Sprite::new(Vec2::new(32.0, 32.0)),
                ..Default::default()
            })
            .with(Fireball {
                origin: *origin,
                target: target_2,
            })
            .with(Collider::Projectile);
    }
}
pub struct Basic;

impl Attack for Basic {
    fn attack(
        self: &Self,
        commands: &mut Commands,
        origin: &Vec3,
        target: &Vec3,
        fire_sp: &Handle<ColorMaterial>,
    ) {
        commands
            .spawn(SpriteBundle {
                material: fire_sp.clone(),
                transform: Transform::from_translation(*origin),
                sprite: Sprite::new(Vec2::new(32.0, 32.0)),
                ..Default::default()
            })
            .with(Fireball {
                origin: *origin,
                target: *target,
            })
            .with(Collider::Projectile);
    }
}

pub trait Attack {
    fn attack(
        self: &Self,
        commands: &mut Commands,
        origin: &Vec3,
        target: &Vec3,
        fire_sp: &Handle<ColorMaterial>,
    );
}
