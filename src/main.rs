use core::f32;

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

// #[macro_use]
extern crate simplelog;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};

const WIN_SIZE: (f32, f32) = (300.0, 300.0);
const TEX_SIZE: f32 = 16.0;

fn main() {
    //set up logging
    TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap();

    let mut timer = FireballTimer(Timer::from_seconds(0.1, true));
    timer.0.pause();
    timer.0.reset();
    App::build()
        .add_resource(WindowDescriptor {
            title: "Game Thing".to_string(),
            width: 600.0,
            height: 600.0,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .init_resource::<MousePos>()
        .add_plugins(DefaultPlugins)
        .add_resource(ClearColor(Color::rgb(25.0, 25.0, 50.0)))
        .add_resource(timer)
        .add_startup_system(setup.system())
        .add_system(move_sys.system())
        .add_system(collide_player.system())
        .add_system(spawn_fireball.system())
        .add_system(mouse_sys.system())
        .add_system(move_fireball.system())
        .add_system(grab_cursor.system())
        .add_system(spawner_animate.system())
        .add_system(spawn_enemies.system())
        .add_system(move_enemies.system())
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

struct FireballTimer(Timer);
struct EnemyTimer(Timer);

//set up assets and stuff
fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let kerb = asset_server.load("kerbee.png");
    let fireball = asset_server.load("fireball.png");
    let reticle = asset_server.load("reticle.png");
    let enemy = asset_server.load("enemy.png");

    let spawner = asset_server.load("spawner.png");
    let spawner_atlas = TextureAtlas::from_grid(spawner, Vec2::new(32.0, 32.0), 3, 1);
    let mut spawner_transform = Transform::from_scale(Vec3::splat(2.0));
    spawner_transform.translation.x = -200.0;
    spawner_transform.translation.y = -200.0;

    let fireball_handle = materials.add(fireball.into());
    let enemy_handle = materials.add(enemy.into());

    commands
        .spawn(Camera2dBundle::default())
        .with(MainCamera)
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
        .spawn(SpriteBundle {
            material: materials.add(reticle.into()),
            transform: Transform::default(),
            sprite: Sprite::new(Vec2::new(32.0, 32.0)),
            ..Default::default()
        })
        .with(Reticle)
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlases.add(spawner_atlas.into()),
            transform: spawner_transform,
            ..Default::default()
        })
        .with(Sprite::new(Vec2::new(64.0, 64.0)))
        .with(Timer::from_seconds(0.12, true))
        .with(EnemySpawn)
        .with(EnemyTimer(Timer::from_seconds(5.0, true)))
        .with(Collider::Solid)
        .insert_resource(FireballSpr(fireball_handle))
        .insert_resource(EnemySpr(enemy_handle));
}

//move the sprite
fn move_sys(
    mouse_pos: Res<MousePos>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut q: Query<(&mut Player, &mut Transform)>,
    mut ret: Query<&mut Transform, With<Reticle>>,
) {
    for (mut p, mut transform) in q.iter_mut() {
        let mut x_dir = 0.0;
        let mut y_dir = 0.0;
        let mut sprint = 1.0;

        if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
            x_dir -= 1.0;
        }

        if input.pressed(KeyCode::D) || input.pressed(KeyCode::Right) {
            x_dir += 1.0;
        }

        if input.pressed(KeyCode::W) || input.pressed(KeyCode::Up) {
            y_dir += 1.0;
        }

        if input.pressed(KeyCode::S) || input.pressed(KeyCode::Down) {
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
            .min(WIN_SIZE.0 - TEX_SIZE)
            .max(-(WIN_SIZE.0 - TEX_SIZE));
        translation.y = translation
            .y
            .min(WIN_SIZE.0 - TEX_SIZE)
            .max(-(WIN_SIZE.0 - TEX_SIZE));

        //move the reticle
        let ret_pos = &mut ret.iter_mut().next().unwrap().translation;
        let ret_line = (mouse_pos.0.translation - *translation).normalize();

        ret_pos.x = translation.x + (150.0 * ret_line.x);
        ret_pos.y = translation.y + (150.0 * ret_line.y);
    }
}

fn grab_cursor(
    mut windows: ResMut<Windows>,
    key: Res<Input<KeyCode>>,
    btn: Res<Input<MouseButton>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if key.pressed(KeyCode::Back) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
    }
}

fn mouse_sys(
    ev_cursor: Res<Events<CursorMoved>>,
    mut evr_cursor: Local<EventReader<CursorMoved>>,
    wnds: Res<Windows>,
    mut pos: ResMut<MousePos>,
    q_camera: Query<&Transform, With<MainCamera>>,
    player: Query<&Transform, With<Player>>,
) {
    // assuming there is exactly one main camera entity, so this is OK
    let camera_transform = q_camera.iter().next().unwrap();

    for ev in evr_cursor.iter(&ev_cursor) {
        let wnd = wnds.get(ev.id).unwrap();

        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        let p = ev.position - size / 2.0;

        //convert the screen coords to world coords
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

        let player_pos = player.iter().next().unwrap().translation;

        let translation = &mut pos.0.translation;
        translation.x = pos_wld.x + player_pos.x;
        translation.y = pos_wld.y + player_pos.y;
    }
}

//spawn a fireball while the left mouse button is held down, on a 0.1s timer
fn spawn_fireball(
    commands: &mut Commands,
    input: Res<Input<MouseButton>>,
    pos: Res<MousePos>,
    fire_sp: Res<FireballSpr>,
    player: Query<&Transform, With<Player>>,
    time: Res<Time>,
    mut timer: ResMut<FireballTimer>,
) {
    if !timer.0.tick(time.delta_seconds()).just_finished() && !timer.0.paused() {
        return;
    }

    if input.pressed(MouseButton::Left) {
        timer.0.unpause();

        for transform in player.iter() {
            let origin = transform.translation;
            let target = {
                // let dir = ((pos.0.translation - origin) / (pos.0.translation - origin)).abs();

                // let res = Vec3::new(
                //     pos.0.translation.x + (1.0 * dir.x),
                //     pos.0.translation.y + (1.0 * dir.y),
                //     0.0,
                // );
                // println!("{:?}", res);
                // res
                debug!("Fireball target: {}", pos.0.translation);
                pos.0.translation
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
        if translation.x >= 400.0
            || translation.x <= -400.0
            || translation.y >= 400.0
            || translation.y <= -400.0
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
                .with(Enemy { speed: 200.0 })
                .with(Collider::Enemy);
        }
    }
}

//--collision systems--//
fn collide_player(
    commands: &mut Commands,
    mut q: Query<(&mut Player, &mut Transform, &Sprite)>,
    collision_q: Query<(Entity, &Sprite, &Transform, &Collider)>,
) {
    for (mut p, mut player_t, player_s) in q.iter_mut() {
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
                            info!("player pos {}", player_t.translation);
                            info!("collider pos {}", tr.translation);
                            player_t.translation.y -= ((player_t.translation.y - (player_s.size.y*0.5)) - (tr.translation.y + (spr.size.y*0.5)));
                            player_t.translation.y = player_t.translation.y.floor();
                            info!("new pos {}", player_t.translation);
                        }
                        Collision::Bottom => {
                            info!("Bottom collision");
                        }
                        Collision::Left => {
                            info!("Left collision");
                        }
                        Collision::Right => {
                            info!("Right collision");
                        }
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
