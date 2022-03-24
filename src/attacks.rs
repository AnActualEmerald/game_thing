use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Color, Commands, Handle, Image, Sprite, SpriteBundle, Transform};
use log::{debug, error, info};

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
        fire_sp: &Handle<Image>,
    ) {
        let up: Vec3 = Vec3::Y * 100.0;
        let raw_target = *target - *origin;
        let angle = up.angle_between(raw_target) + 90f32.to_radians();

        let diff: Vec3 =
            Vec3::new(angle.cos() + angle.sin(), angle.sin() - angle.cos(), 0.0).normalize() * 10.0;

        let target_1 = Vec3::new(target.x + diff.x, target.y + diff.y, 0.0);
        let target_2 = Vec3::new(target.x - diff.x, target.y - diff.y, 0.0);

        commands
            .spawn_bundle(SpriteBundle {
                texture: fire_sp.clone(),
                transform: Transform::from_translation(*origin),
                ..Default::default()
            })
            .insert(Fireball {
                origin: *origin,
                target: target_1,
            })
            .insert(Collider::Projectile);

        commands
            .spawn_bundle(SpriteBundle {
                texture: fire_sp.clone(),
                transform: Transform::from_translation(*origin),
                ..Default::default()
            })
            .insert(Fireball {
                origin: *origin,
                target: target_2,
            })
            .insert(Collider::Projectile);

        commands
            .spawn_bundle(SpriteBundle {
                texture: fire_sp.clone(),
                transform: Transform::from_translation(*origin),
                ..Default::default()
            })
            .insert(Fireball {
                origin: *origin,
                target: *target,
            })
            .insert(Collider::Projectile);
    }
}
pub struct Basic;

impl Attack for Basic {
    fn attack(
        self: &Self,
        commands: &mut Commands,
        origin: &Vec3,
        target: &Vec3,
        fire_sp: &Handle<Image>,
    ) {
        commands
            .spawn_bundle(SpriteBundle {
                texture: fire_sp.clone(),
                transform: Transform::from_translation(*origin),
                ..Default::default()
            })
            .insert(Fireball {
                origin: *origin,
                target: *target,
            })
            .insert(Collider::Projectile);
    }
}

pub trait Attack {
    fn attack(
        self: &Self,
        commands: &mut Commands,
        origin: &Vec3,
        target: &Vec3,
        fire_sp: &Handle<Image>,
    );
}
