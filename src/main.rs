use attacks::Attack;
use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use log::{debug, error, info, trace, warn};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::fs::File;

mod attacks;
mod ui;
mod gameplay;

use gameplay::*;

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
        .add_system(ui::player_hit_handler.system())
        .run();
}

//--components--//

struct MainCamera;

pub enum Collider {
    Solid,
    Enemy,
    Projectile,
}

//used in ui module
pub struct Index(i32);

//--events--//
//these need to be public for use in other files

pub struct PlayerHitEvent(Entity);

//--resources--//

pub struct FireballSpr(Handle<ColorMaterial>);
pub struct EnemySpr(Handle<ColorMaterial>);
#[derive(Default)]
struct MousePos(Transform);
#[derive(Default)]
struct MouseDelta(Vec2);

pub struct FireballTimer(Timer);
pub struct EnemyTimer(Timer);

pub struct DifficultyTimer(Timer);

pub struct CurrentAttack(
    Box<dyn Attack + Send + Sync>, // Box<dyn FnMut(&mut Commands, &Vec3, &Vec3, &Handle<ColorMaterial>) + Send + Sync>,
);

//--systems--//

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
    let heart = asset_server.load("heart.png");

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
        .with(Player::new(200.0))
        .insert_resource(FireballSpr(fireball_handle))
        .insert_resource(EnemySpr(enemy_handle))
        .insert_resource(Events::<PlayerHitEvent>::default())
        .insert_resource(DifficultyTimer(Timer::from_seconds(30.0, true)))
        .insert_resource(CurrentAttack(Box::new(attacks::Basic)));

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

    //add hearts
    for i in 0..3 {
        let heart_atlas = TextureAtlas::from_grid(heart.clone_weak(), Vec2::new(16.0, 16.0), 2, 1);
        let mut tr = Transform::from_translation(Vec3::new(
            -WIN_SIZE.0 + (36.0 * i as f32) + 20.0,
            WIN_SIZE.1 - 20.0,
            0.0,
        ));
        tr.scale = Vec3::splat(2.0);
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: texture_atlases.add(heart_atlas.into()),
                transform: tr,
                ..Default::default()
            })
            .with(Index(i));
    }
    audio.play(music);
    info!("Game start :)");
}


fn mouse_sys(
    ev_cursor: Res<Events<CursorMoved>>,
    mut evr_cursor: Local<EventReader<CursorMoved>>,
    mut wnds: ResMut<Windows>,
    mut pos: ResMut<MousePos>,
    mut delta: ResMut<MouseDelta>,
    q_camera: Query<&Transform, With<MainCamera>>,
    player: Query<&Transform, With<gameplay::Player>>,
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



//--collision systems--//
fn collide_player(
    commands: &mut Commands,
    mut q: Query<(Entity, &mut Transform, &Sprite), With<Player>>,
    collision_q: Query<(Entity, &Sprite, &Transform, &Collider)>,
    mut ev_playerhit: ResMut<Events<PlayerHitEvent>>,
) {
    for (player_ent, mut player_t, player_s) in q.iter_mut() {
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
                    ev_playerhit.send(PlayerHitEvent(player_ent));
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
