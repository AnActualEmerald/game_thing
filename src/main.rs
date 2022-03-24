use attacks::Attack;
use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use log::{debug, error, info, trace, warn};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::fs::File;

mod attacks;
mod gameplay;
mod ui;

use gameplay::*;

const WIN_SIZE: (f32, f32) = (1280.0 / 2.0, 720.0 / 2.0);
const TEX_SIZE: f32 = 16.0;

fn main() {
    //set up logging
    // let err = CombinedLogger::init(vec![
    //     #[cfg(debug_assertions)]
    //     TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
    //     #[cfg(not(debug_assertions))]
    //     simplelog::WriteLogger::new(
    //         LevelFilter::Info,
    //         Config::default(),
    //         File::create(format!(
    //             "game_{}.log",
    //             chrono::Local::now().date().format("%m_%d_%y")
    //         ))
    //         .unwrap(),
    //     ),
    //     #[cfg(not(debug_assertions))]
    //     simplelog::WriteLogger::new(
    //         LevelFilter::Trace,
    //         Config::default(),
    //         File::create(format!(
    //             "debug_{}.log",
    //             chrono::Local::now().date().format("%m_%d_%y")
    //         ))
    //         .unwrap(),
    //     ),
    // ]);

    // println!("Logging error {:?}", err);

    let mut timer = FireballTimer(Timer::from_seconds(0.1, true));
    timer.0.pause();
    timer.0.reset();
    App::new()
        .insert_resource(WindowDescriptor {
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
        .insert_resource(ClearColor(Color::rgb(25.0, 25.0, 50.0)))
        .insert_resource(timer)
        .add_event::<PlayerHitEvent>()
        .add_startup_system(setup)
        .add_system(move_sys)
        // .add_system(collide_player)
        .add_system(spawn_fireball)
        .add_system(mouse_sys)
        .add_system(move_fireball)
        .add_system(spawner_animate)
        .add_system(spawn_enemies)
        .add_system(move_enemies)
        // .add_system(collide_enemies)
        // .add_system(collide_fireballs)
        .add_system(ui::player_hit_handler)
        .run();
}

//--components--//

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
pub enum Collider {
    Solid,
    Enemy,
    Projectile,
}

//Hitbox with a size
#[derive(Component)]
pub struct Hitbox(Vec2);

//used in ui module
#[derive(Component)]
pub struct Index(i32);

//--events--//
//these need to be public for use in other files

pub struct PlayerHitEvent(Entity);

//--resources--//

pub struct FireballSpr(Handle<Image>);
pub struct EnemySpr(Handle<Image>);
#[derive(Default)]
struct MousePos(Transform);
#[derive(Default)]
struct MouseDelta(Vec2);

pub struct FireballTimer(Timer);

#[derive(Component)]
pub struct EnemyTimer(Timer);

pub struct DifficultyTimer(Timer);

pub struct CurrentAttack(
    Box<dyn Attack + Send + Sync>, // Box<dyn FnMut(&mut Commands, &Vec3, &Vec3, &Handle<ColorMaterial>) + Send + Sync>,
);

//--systems--//

//set up assets and stuff
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands
        .spawn_bundle(SpriteBundle {
            texture: reticle,
            transform: Transform {
                translation: Vec3::new(100.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Reticle);
    commands
        .spawn_bundle(SpriteBundle {
            texture: kerb.into(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player::new(200.0));
    commands.insert_resource(FireballSpr(fireball));
    commands.insert_resource(EnemySpr(enemy));
    commands.insert_resource(DifficultyTimer(Timer::from_seconds(30.0, true)));
    commands.insert_resource(CurrentAttack(Box::new(attacks::Split)));

    let spawner_atlas = TextureAtlas::from_grid(spawner, Vec2::new(32.0, 32.0), 3, 1);
    let spawner_handle = texture_atlases.add(spawner_atlas.into());
    //add spawners
    for x in -1..2 {
        for y in -1..2 {
            if x == 0 || y == 0 {
                continue;
            }
            spawner_transform.translation.x = (WIN_SIZE.0 - 100.0) * x as f32;
            spawner_transform.translation.y = (WIN_SIZE.1 - 100.0) * y as f32;

            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: spawner_handle.clone(),
                    transform: spawner_transform,
                    ..Default::default()
                })
                .insert(Timer::from_seconds(0.12, true))
                .insert(EnemySpawn)
                .insert(EnemyTimer(Timer::from_seconds(2.0, true)))
                .insert(Collider::Solid);
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
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlases.add(heart_atlas.into()),
                transform: tr,
                ..Default::default()
            })
            .insert(Index(i));
    }
    audio.play(music);
    info!("Game start :)");
}

fn mouse_sys(
    mut ev_cursor: EventReader<CursorMoved>,
    mut wnds: ResMut<Windows>,
    mut pos: ResMut<MousePos>,
    mut delta: ResMut<MouseDelta>,
    q_camera: Query<&Transform, With<MainCamera>>,
    player: Query<&Transform, With<gameplay::Player>>,
) {
    // assuming there is exactly one main camera entity, so this is OK
    let camera_transform = q_camera.iter().next().unwrap();
    let start = pos.0;
    for ev in ev_cursor.iter() {
        let wnd = wnds.get_mut(ev.id).unwrap();

        let custom_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        let p = ev.position - custom_size / 2.0;

        //convert the screen coords to world coords
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

        let player_pos = player.iter().next().unwrap().translation;

        let translation = &mut pos.0.translation;

        translation.x = pos_wld.x + player_pos.x;
        translation.y = pos_wld.y + player_pos.y;

        let res = pos.0.translation - start.translation;

        delta.0 = Vec2::new(delta.0.x + res.x, delta.0.y + res.y);
    }
}

//move enemies towards the player
fn move_enemies(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut Transform, &Enemy), Without<Player>>,
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
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &Fireball, &mut Transform)>,
) {
    for (e, f, mut current) in q.iter_mut() {
        current.rotate(Quat::from_rotation_z(0.5));
        let translation = &mut current.translation;
        let direction = (f.target - f.origin).normalize();
        translation.x += 500.0 * direction.x * time.delta_seconds();
        translation.y += 500.0 * direction.y * time.delta_seconds();
        //if the fireball goes off screen, remove it
        if translation.x >= WIN_SIZE.0 + 100.0
            || translation.x <= -WIN_SIZE.0 - 100.0
            || translation.y >= WIN_SIZE.1 + 100.0
            || translation.y <= -WIN_SIZE.1 - 100.0
        {
            commands.entity(e).despawn();
            debug!("Removed fireball")
        }
    }
}

//--collision systems--//
fn collide_player(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transform, &Sprite), With<Player>>,
    collision_q: Query<(Entity, &Sprite, &Transform, &Collider), Without<Player>>,
    mut ev_playerhit: EventWriter<PlayerHitEvent>,
) {
    for (player_ent, mut player_t, player_s) in q.iter_mut() {
        for (ent, spr, tr, col) in collision_q.iter() {
            let collision = collide(
                player_t.translation,
                player_s.custom_size.unwrap(),
                tr.translation,
                spr.custom_size.unwrap(),
            );
            if let Some(collision) = collision {
                if let Collider::Enemy = *col {
                    commands.entity(ent).despawn();
                    ev_playerhit.send(PlayerHitEvent(player_ent));
                    info!("player got hit");
                }

                //refers to the side of the object being collided with, not the player
                if let Collider::Solid = *col {
                    match collision {
                        Collision::Top => {
                            player_t.translation.y -= (player_t.translation.y
                                - (player_s.custom_size.unwrap().y * 0.5))
                                - (tr.translation.y + (spr.custom_size.unwrap().y * 0.5));
                            player_t.translation.y = player_t.translation.y.floor();
                        }
                        Collision::Bottom => {
                            player_t.translation.y -= (player_t.translation.y
                                + (player_s.custom_size.unwrap().y * 0.5))
                                - (tr.translation.y - (spr.custom_size.unwrap().y * 0.5));
                            player_t.translation.y = player_t.translation.y.floor();
                        }
                        Collision::Left => {
                            player_t.translation.x -= (player_t.translation.x
                                + (player_s.custom_size.unwrap().x * 0.5))
                                - (tr.translation.x - (spr.custom_size.unwrap().x * 0.5));
                            player_t.translation.x = player_t.translation.x.floor();
                        }
                        Collision::Right => {
                            player_t.translation.x -= (player_t.translation.x
                                - (player_s.custom_size.unwrap().x * 0.5))
                                - (tr.translation.x + (spr.custom_size.unwrap().x * 0.5));
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
                spr.custom_size.unwrap(),
                other_tr.translation,
                other_spr.custom_size.unwrap(),
            );

            if let Some(coll) = collision {
                match coll {
                    Collision::Top => {
                        tr.translation.y -= (tr.translation.y - (spr.custom_size.unwrap().y * 0.5))
                            - (other_tr.translation.y + (other_spr.custom_size.unwrap().y * 0.5));
                        tr.translation.y = tr.translation.y.ceil();
                    }
                    Collision::Bottom => {
                        tr.translation.y -= (tr.translation.y + (spr.custom_size.unwrap().y * 0.5))
                            - (other_tr.translation.y - (other_spr.custom_size.unwrap().y * 0.5));
                        tr.translation.y = tr.translation.y.ceil();
                    }
                    Collision::Left => {
                        tr.translation.x -= (tr.translation.x + (spr.custom_size.unwrap().x * 0.5))
                            - (other_tr.translation.x - (other_spr.custom_size.unwrap().x * 0.5));
                        tr.translation.x = tr.translation.x.ceil();
                    }
                    Collision::Right => {
                        tr.translation.x -= (tr.translation.x - (spr.custom_size.unwrap().x * 0.5))
                            - (other_tr.translation.x + (other_spr.custom_size.unwrap().x * 0.5));
                        tr.translation.x = tr.translation.x.ceil();
                    }
                }
            }
        }
    }
}

fn collide_fireballs(
    mut commands: Commands,
    balls: Query<(Entity, &Transform, &Sprite), With<Fireball>>,
    col_query: Query<(Entity, &Transform, &Collider, &Sprite), Without<Player>>,
) {
    for (ball_ent, ball_tr, ball_spr) in balls.iter() {
        for (ent, tr, col, spr) in col_query.iter() {
            if let Some(_) = collide(
                ball_tr.translation,
                ball_spr.custom_size.unwrap(),
                tr.translation,
                spr.custom_size.unwrap(),
            ) {
                match *col {
                    Collider::Enemy => {
                        commands.entity(ball_ent).despawn();
                        commands.entity(ent).despawn();
                    }
                    Collider::Solid => {
                        commands.entity(ball_ent).despawn();
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
        timer.tick(time.delta());
        if timer.finished() {
            let atlas = sheets.get(handle).unwrap();
            sprite.index = (sprite.index + 1) % atlas.textures.len();
        }
    }
}
