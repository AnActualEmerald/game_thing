use core::f32;

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use log::{debug, error, info, trace, warn};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::fs::File;

const WIN_SIZE: (f32, f32) = (1280.0 / 2.0, 720.0 / 2.0);
const TEX_SIZE: f32 = 16.0;

fn main() {
    //set up logging
    CombinedLogger::init(vec![
        #[cfg(debug_assertions)]
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        #[cfg(not(debug_assertions))]
        simplelog::WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(format!(
                "game_{}.log",
                chrono::Local::now().date().format("%m_%d_%y")
            ))
            .unwrap(),
        ),
        #[cfg(not(debug_assertions))]
        simplelog::WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(format!(
                "debug_{}.log",
                chrono::Local::now().date().format("%m_%d_%y")
            ))
            .unwrap(),
        ),
    ])
    .unwrap();

    let mut timer = FireballTimer(Timer::from_seconds(0.1, true));
    timer.0.pause();
    timer.0.reset();
    App::build()
        .add_resource(WindowDescriptor {
            title: "Game Thing".to_string(),
            width: 1280.0,
            height: 720.0,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .init_resource::<MousePos>()
        .init_resource::<MouseDelta>()
        .add_plugins(DefaultPlugins)
        .add_resource(ClearColor(Color::rgb(25.0, 25.0, 50.0)))
        .add_resource(timer)
        .add_startup_system(setup.system())
        .add_system(move_sys.system())
        .add_system(collide_player.system())
        .add_system(spawn_fireball.system())
        .add_system(mouse_sys.system())
        .add_system(move_fireball.system())
        .add_system(spawner_animate.system())
        .add_system(spawn_enemies.system())
        .add_system(move_enemies.system())
        .add_system(collide_enemies.system())
        .add_system(collide_fireballs.system())
        .run();
}

//--components--//

struct Player {
    speed: f32,
    mod_y: f32,
    mod_x: f32,
}

struct Fireball {
    origin: Vec3,
    target: Vec3,
}

struct Enemy {
    speed: f32,
}

struct MainCamera;
struct Reticle;
struct EnemySpawn;

enum Collider {
    Solid,
    Enemy,
    Projectile,
}

//--resources--//

struct FireballSpr(Handle<ColorMaterial>);
struct EnemySpr(Handle<ColorMaterial>);
#[derive(Default)]
struct MousePos(Transform);
#[derive(Default)]
struct MouseDelta(Vec2);

struct FireballTimer(Timer);
struct EnemyTimer(Timer);

//set up assets and stuff
fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    audio: Res<Audio>,
) {
    let music = asset_server.load("music1.mp3");
    let kerb = asset_server.load("kerbee.png");
    let fireball = asset_server.load("fireball.png");
    let reticle = asset_server.load("reticle.png");
    let enemy = asset_server.load("enemy.png");

    let spawner = asset_server.load("spawner.png");
    let mut spawner_transform = Transform::from_scale(Vec3::splat(2.0));
    let fireball_handle = materials.add(fireball.into());
    let enemy_handle = materials.add(enemy.into());

    commands
        .spawn(Camera2dBundle::default())
        .with(MainCamera)
        .spawn(SpriteBundle {
            material: materials.add(reticle.into()),
            transform: Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(32.0, 32.0)),
            ..Default::default()
        })
        .with(Reticle)
        .spawn(SpriteBundle {
            material: materials.add(kerb.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(Player {
            speed: 200.0,
            mod_x: 0.0,
            mod_y: 0.0,
        })
        .insert_resource(FireballSpr(fireball_handle))
        .insert_resource(EnemySpr(enemy_handle));

    //add spawners
    for x in -1..2 {
        for y in -1..2 {
            if x == 0 || y == 0 {
                continue;
            }
            spawner_transform.translation.x = (WIN_SIZE.0 - 100.0) * x as f32;
            spawner_transform.translation.y = (WIN_SIZE.1 - 100.0) * y as f32;
            let spawner_atlas =
                TextureAtlas::from_grid(spawner.clone_weak(), Vec2::new(32.0, 32.0), 3, 1);
            commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: texture_atlases.add(spawner_atlas.into()),
                    transform: spawner_transform,
                    ..Default::default()
                })
                .with(Sprite::new(Vec2::new(64.0, 64.0)))
                .with(Timer::from_seconds(0.12, true))
                .with(EnemySpawn)
                .with(EnemyTimer(Timer::from_seconds(2.0, true)))
                .with(Collider::Solid);
            info!("Added enemy spawn at {}", spawner_transform.translation);
        }
    }
    audio.play(music);
    info!("Game start :)");
}

//move the sprite
fn move_sys(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut q: Query<(&Player, &mut Transform)>,
    mut ret: Query<&mut Transform, With<Reticle>>,
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

fn mouse_sys(
    ev_cursor: Res<Events<CursorMoved>>,
    mut evr_cursor: Local<EventReader<CursorMoved>>,
    mut wnds: ResMut<Windows>,
    mut pos: ResMut<MousePos>,
    mut delta: ResMut<MouseDelta>,
    q_camera: Query<&Transform, With<MainCamera>>,
    player: Query<&Transform, With<Player>>,
) {
    // assuming there is exactly one main camera entity, so this is OK
    let camera_transform = q_camera.iter().next().unwrap();
    let start = pos.0;
    for ev in evr_cursor.iter(&ev_cursor) {
        let wnd = wnds.get_mut(ev.id).unwrap();

        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        let p = ev.position - size / 2.0;

        //convert the screen coords to world coords
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

        let player_pos = player.iter().next().unwrap().translation;

        let translation = &mut pos.0.translation;

        translation.x = pos_wld.x + player_pos.x;
        translation.y = pos_wld.y + player_pos.y;

        delta.0 = delta.0 + (pos.0.translation - start.translation).into();
    }
}

//spawn a fireball while the left mouse button is held down, on a 0.1s timer
fn spawn_fireball(
    commands: &mut Commands,
    input: Res<Input<KeyCode>>,
    fire_sp: Res<FireballSpr>,
    player: Query<&Transform, With<Player>>,
    ret: Query<&Transform, With<Reticle>>,
    time: Res<Time>,
    mut timer: ResMut<FireballTimer>,
) {
    if !timer.0.tick(time.delta_seconds()).just_finished() && !timer.0.paused() {
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
                let tr = ret.iter().next().unwrap_or_else(|| {
                    error!("Tried to aim at the reticle, but the reticle was missing");
                    panic!("Expected a reticle, got NONE instead");
                });
                debug!("Fireball target: {}", tr.translation);
                tr.translation
            };

            commands
                .spawn(SpriteBundle {
                    material: fire_sp.0.clone(),
                    transform: *transform,
                    sprite: Sprite::new(Vec2::new(32.0, 32.0)),
                    ..Default::default()
                })
                .with(Fireball {
                    origin: origin,
                    target: target,
                })
                .with(Collider::Projectile);
        }
    } else {
        timer.0.pause();
        timer.0.reset();
    }
}

//move enemies towards the player
fn move_enemies(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut Transform, &Enemy)>,
) {
    //there should only be one player
    let player = player_query.iter().next().unwrap();

    for (mut transform, enemy) in enemies.iter_mut() {
        let move_vec = (player.translation - transform.translation).normalize();

        transform.translation.x += move_vec.x * enemy.speed * time.delta_seconds();
        transform.translation.y += move_vec.y * enemy.speed * time.delta_seconds();
    }
}

//move active fireballs towards their target and despawn any that go off screen
fn move_fireball(
    commands: &mut Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &Fireball, &mut Transform)>,
) {
    for (e, f, mut current) in q.iter_mut() {
        current.rotate(Quat::from_rotation_z(0.5));
        let mut translation = &mut current.translation;
        let direction = (f.target - f.origin).normalize();
        translation.x += 500.0 * direction.x * time.delta_seconds();
        translation.y += 500.0 * direction.y * time.delta_seconds();
        //if the fireball goes off screen, remove it
        if translation.x >= WIN_SIZE.0 + 100.0
            || translation.x <= -WIN_SIZE.0 - 100.0
            || translation.y >= WIN_SIZE.1 + 100.0
            || translation.y <= -WIN_SIZE.1 - 100.0
        {
            commands.despawn(e);
            debug!("Removed fireball")
        }
    }
}

//spawn enemies from each active spawner
fn spawn_enemies(
    commands: &mut Commands,
    time: Res<Time>,
    enemy: Res<EnemySpr>,
    mut q: Query<(&Transform, &mut EnemyTimer)>,
) {
    for (transform, mut timer) in q.iter_mut() {
        if timer.0.tick(time.delta_seconds()).finished() {
            commands
                .spawn(SpriteBundle {
                    material: enemy.0.clone(),
                    transform: *transform,
                    sprite: Sprite::new(Vec2::new(14.0, 16.0)),
                    ..Default::default()
                })
                .with(Enemy { speed: 175.0 })
                .with(Collider::Enemy);
        }
    }
}

//--collision systems--//
fn collide_player(
    commands: &mut Commands,
    mut q: Query<(&mut Transform, &Sprite), With<Player>>,
    collision_q: Query<(Entity, &Sprite, &Transform, &Collider)>,
) {
    for (mut player_t, player_s) in q.iter_mut() {
        for (ent, spr, tr, col) in collision_q.iter() {
            let collision = collide(
                player_t.translation,
                player_s.size,
                tr.translation,
                spr.size,
            );
            if let Some(collision) = collision {
                if let Collider::Enemy = *col {
                    commands.despawn(ent);
                    info!("player got hit");
                }

                //refers to the side of the object being collided with, not the player
                if let Collider::Solid = *col {
                    match collision {
                        Collision::Top => {
                            player_t.translation.y -= (player_t.translation.y
                                - (player_s.size.y * 0.5))
                                - (tr.translation.y + (spr.size.y * 0.5));
                            player_t.translation.y = player_t.translation.y.floor();
                        }
                        Collision::Bottom => {
                            player_t.translation.y -= (player_t.translation.y
                                + (player_s.size.y * 0.5))
                                - (tr.translation.y - (spr.size.y * 0.5));
                            player_t.translation.y = player_t.translation.y.floor();
                        }
                        Collision::Left => {
                            player_t.translation.x -= (player_t.translation.x
                                + (player_s.size.x * 0.5))
                                - (tr.translation.x - (spr.size.x * 0.5));
                            player_t.translation.x = player_t.translation.x.floor();
                        }
                        Collision::Right => {
                            player_t.translation.x -= (player_t.translation.x
                                - (player_s.size.x * 0.5))
                                - (tr.translation.x + (spr.size.x * 0.5));
                            player_t.translation.x = player_t.translation.x.floor();
                        }
                    }
                }
            }
        }
    }
}

fn collide_enemies(mut q: Query<(&mut Transform, &Sprite), With<Enemy>>) {
    let mut enemies = q.iter_mut();
    while let Some((mut tr, spr)) = enemies.next() {
        if let Some((other_tr, other_spr)) = enemies.next() {
            let collision = collide(
                tr.translation,
                spr.size,
                other_tr.translation,
                other_spr.size,
            );

            if let Some(coll) = collision {
                match coll {
                    Collision::Top => {
                        tr.translation.y -= (tr.translation.y - (spr.size.y * 0.5))
                            - (other_tr.translation.y + (other_spr.size.y * 0.5));
                        tr.translation.y = tr.translation.y.ceil();
                    }
                    Collision::Bottom => {
                        tr.translation.y -= (tr.translation.y + (spr.size.y * 0.5))
                            - (other_tr.translation.y - (other_spr.size.y * 0.5));
                        tr.translation.y = tr.translation.y.ceil();
                    }
                    Collision::Left => {
                        tr.translation.x -= (tr.translation.x + (spr.size.x * 0.5))
                            - (other_tr.translation.x - (other_spr.size.x * 0.5));
                        tr.translation.x = tr.translation.x.ceil();
                    }
                    Collision::Right => {
                        tr.translation.x -= (tr.translation.x - (spr.size.x * 0.5))
                            - (other_tr.translation.x + (other_spr.size.x * 0.5));
                        tr.translation.x = tr.translation.x.ceil();
                    }
                }
            }
        }
    }
}

fn collide_fireballs(
    commands: &mut Commands,
    balls: Query<(Entity, &Transform, &Sprite), With<Fireball>>,
    col_query: Query<(Entity, &Transform, &Collider, &Sprite)>,
) {
    for (ball_ent, ball_tr, ball_spr) in balls.iter() {
        for (ent, tr, col, spr) in col_query.iter() {
            if let Some(_) = collide(ball_tr.translation, ball_spr.size, tr.translation, spr.size) {
                match *col {
                    Collider::Enemy => {
                        commands.despawn(ball_ent).despawn(ent);
                    }
                    Collider::Solid => {
                        commands.despawn(ball_ent);
                    }
                    _ => continue,
                }
            }
        }
    }
}

//--animation systems--//

//animate the enemy spawners
fn spawner_animate(
    time: Res<Time>,
    sheets: Res<Assets<TextureAtlas>>,
    mut q: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, handle) in q.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            let atlas = sheets.get(handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % atlas.textures.len()) as u32;
        }
    }
}
