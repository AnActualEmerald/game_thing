use bevy::prelude::*;
use log::{debug, error, info, trace, warn};
use rand::random;
use std::time::Duration;

use crate::{
    attacks::{self, Attack},
    Collider, CurrentAttack, DifficultyTimer, EnemySpr, EnemyTimer, FireballSpr, FireballTimer,
    TEX_SIZE, WIN_SIZE,
};

#[derive(Component)]
pub struct Powerup {
    attack: Box<dyn Attack + Send + Sync>,
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    mod_y: f32,
    mod_x: f32,
}

impl Player {
    pub fn new(speed: f32) -> Player {
        Player {
            speed: speed,
            mod_x: 0.0,
            mod_y: 0.0,
        }
    }
}

#[derive(Component)]
pub struct Fireball {
    pub origin: Vec3,
    pub target: Vec3,
}

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
}

#[derive(Component)]
pub struct Reticle;

#[derive(Component)]
pub struct EnemySpawn;

#[derive(Default)]
pub struct Elapsed(f32);

//move the sprite
pub fn move_sys(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut q: Query<(&Player, &mut Transform)>,
    mut ret: Query<&mut Transform, (With<Reticle>, Without<Player>)>,
) {
    for (p, mut transform) in q.iter_mut() {
        let mut x_dir = 0.0;
        let mut y_dir = 0.0;
        let mut sprint = 1.0;

        if input.pressed(KeyCode::A) {
            x_dir -= 1.0;
        }

        if input.pressed(KeyCode::D) {
            x_dir += 1.0;
        }

        if input.pressed(KeyCode::W) {
            y_dir += 1.0;
        }

        if input.pressed(KeyCode::S) {
            y_dir -= 1.0;
        }

        if input.pressed(KeyCode::LShift) {
            sprint = 1.5;
        }

        let translation = &mut transform.translation;

        translation.x += time.delta_seconds() * p.speed * (x_dir + p.mod_x) * sprint;
        translation.y += time.delta_seconds() * p.speed * (y_dir + p.mod_y) * sprint;

        //confine player to the screen
        translation.x = translation
            .x
            .min(1280.0 / 2.0 - TEX_SIZE)
            .max(-(1280.0 / 2.0 - TEX_SIZE));
        translation.y = translation
            .y
            .min(720.0 / 2.0 - TEX_SIZE)
            .max(-(720.0 / 2.0 - TEX_SIZE));

        //move the reticle
        let ret_pos = &mut ret.iter_mut().next().unwrap().translation;

        if input.pressed(KeyCode::Up) {
            ret_pos.y = 100.0;
        } else if input.pressed(KeyCode::Down) {
            ret_pos.y = -100.0;
        }

        if input.pressed(KeyCode::Left) {
            ret_pos.x = -100.0;
        } else if input.pressed(KeyCode::Right) {
            ret_pos.x = 100.0;
        }

        if !input.pressed(KeyCode::Up) && !input.pressed(KeyCode::Down) {
            ret_pos.y = 0.0;
        }
        if !input.pressed(KeyCode::Left) && !input.pressed(KeyCode::Right) {
            ret_pos.x = 0.0;
        }

        ret_pos.x += translation.x;
        ret_pos.y += translation.y;
    }
}

//spawn a fireball while the left mouse button is held down, on a 0.1s timer
pub fn spawn_fireball(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    fire_sp: Res<FireballSpr>,
    player: Query<&Transform, With<Player>>,
    ret: Query<&Transform, With<Reticle>>,
    time: Res<Time>,
    mut timer: ResMut<FireballTimer>,
    attack: ResMut<CurrentAttack>,
) {
    if !timer.0.tick(time.delta()).just_finished() && !timer.0.paused() {
        return;
    }

    if input.pressed(KeyCode::Right)
        || input.pressed(KeyCode::Left)
        || input.pressed(KeyCode::Up)
        || input.pressed(KeyCode::Down)
    {
        timer.0.unpause();

        for transform in player.iter() {
            let origin = transform.translation;
            let target = {
                let tr = ret.single();
                debug!("Fireball target: {}", tr.translation);
                tr.translation
            };

            attack.0.attack(&mut commands, &origin, &target, &fire_sp.0);
        }
    } else {
        timer.0.pause();
        timer.0.reset();
    }
}

//spawn enemies from each active spawner
pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    enemy: Res<EnemySpr>,
    mut diff: ResMut<DifficultyTimer>,
    mut q: Query<(&Transform, &mut EnemyTimer)>,
) {
    diff.0.tick(time.delta());
    for (transform, mut timer) in q.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            commands
                .spawn_bundle(SpriteBundle {
                    texture: enemy.0.clone(),
                    transform: transform.with_scale(Vec3::splat(1.5)),
                    sprite: Sprite {
                        color: Color::ALICE_BLUE,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Enemy { speed: 175.0 })
                .insert(Collider::Enemy);
        }
        if diff.0.finished() {
            let dur = timer.0.duration();
            if dur <= Duration::from_secs_f32(0.5) {
                info!("DIFFICULTY MAX!!!");
            } else {
                timer.0.set_duration(dur - Duration::from_secs_f32(0.5));
                info!("Difficulty went up!");
            }
        }
    }
}

pub fn spawn_powerups(
    commands: &mut Commands,
    time: Res<Time>,
    mut elapsed: Local<Elapsed>,
    powerup: Res<EnemySpr>,
) {
    elapsed.0 = elapsed.0 + time.delta_seconds();
    if elapsed.0 >= 30.0 {
        let mut tr = Transform::from_translation(Vec3::new(
            random::<i32>().min(WIN_SIZE.0 as i32) as f32,
            random::<i32>().min(WIN_SIZE.1 as i32) as f32,
            0.0,
        ));
        tr.scale = Vec3::splat(0.5);
        commands
            .spawn_bundle(SpriteBundle {
                texture: powerup.0.clone(),
                transform: tr,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(7.0, 8.0)),
                    color: Color::RED,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Powerup {
                attack: Box::new(attacks::Split),
            });
    }
}
