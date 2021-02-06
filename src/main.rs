use core::f32;

use bevy::{ecs::EntityBuilder, prelude::*, render::camera};

const WIN_SIZE: (f32, f32) = (300.0, 300.0);
const TEX_SIZE: f32 = 16.0;

fn main() {
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
        .add_system(spawn_fireball.system())
        .add_system(mouse_sys.system())
        .add_system(move_fireball.system())
        .add_system(grab_cursor.system())
        .run();
}

//--components--//

struct Player {
    speed: f32,
}

struct Fireball{
  origin: Vec3,
  target: Vec3
}

struct MainCamera;

struct Reticle;

//--resources--//

struct FireballSpr(Handle<ColorMaterial>);
#[derive(Default)]
struct MousePos(Transform);

struct FireballTimer(Timer);


//set up assets and stuff
fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let kerb = asset_server.load("kerbee.png");
    let fireball = asset_server.load("fireball.png");
    let reticle = asset_server.load("reticle.png");

    let fireball_handle = materials.add(fireball.into());

    commands
        .spawn(Camera2dBundle::default())
        .with(MainCamera)
        .spawn(SpriteBundle {
            material: materials.add(kerb.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(Player { speed: 200.0 })
        .spawn(SpriteBundle{
          material: materials.add(reticle.into()),
          transform: Transform::default(),
          ..Default::default()
        })
        .with(Reticle)
        .insert_resource(FireballSpr(fireball_handle));
}

//move the sprite
fn move_sys(time: Res<Time>, input: Res<Input<KeyCode>>, mut q: Query<(&Player, &mut Transform)>) {
    for (p, mut transform) in q.iter_mut() {
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

        translation.x += time.delta_seconds() * p.speed * x_dir * sprint;
        translation.y += time.delta_seconds() * p.speed * y_dir * sprint;

        //confine player to the screen
        translation.x = translation
            .x
            .min(WIN_SIZE.0 - TEX_SIZE)
            .max(-(WIN_SIZE.0 - TEX_SIZE));
        translation.y = translation
            .y
            .min(WIN_SIZE.0 - TEX_SIZE)
            .max(-(WIN_SIZE.0 - TEX_SIZE));

        // println!("x{}, y{}", translation.x, translation.y);
    }
}

fn grab_cursor(mut windows: ResMut<Windows>, key: Res<Input<KeyCode>>){
  let window = windows.get_primary_mut().unwrap();

  if key.pressed(KeyCode::Back) {
    window.set_cursor_lock_mode(false);
    window.set_cursor_visibility(true);
  }else {
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
    mut ret: Query<&mut Transform, With<Reticle>>,
) {

    // assuming there is exactly one main camera entity, so this is OK
    let camera_transform = q_camera.iter().next().unwrap();

    for ev in evr_cursor.iter(&ev_cursor) {
      
      let wnd = wnds.get(ev.id).unwrap();
      
      let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

      let p = ev.position - size / 2.0;  
      
      //convert the screen coords to world coords
      let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

      let translation = &mut pos.0.translation;
        translation.x = pos_wld.x;
        translation.y = pos_wld.y;

        //there should only ever be one of these too
        let reticle_pos =  &mut ret.iter_mut().next().unwrap();
        reticle_pos.translation.x = pos_wld.x;
        reticle_pos.translation.y = pos_wld.y;

    }
}

fn spawn_fireball(
    commands: &mut Commands,
    input: Res<Input<MouseButton>>,
    pos: Res<MousePos>,
    fire_sp: Res<FireballSpr>,
    player: Query<&Transform, With<Player>>,
    time: Res<Time>,
    mut timer: ResMut<FireballTimer>
) {

  if !timer.0.tick(time.delta_seconds()).just_finished() && !timer.0.paused(){
    return;
  }

    if input.pressed(MouseButton::Left) {
      timer.0.unpause();
      for transform in player.iter() {
          commands.spawn(SpriteBundle {
            material: fire_sp.0.clone(),
            transform: transform.clone(),

            ..Default::default()
        }).with(Fireball{
          origin: transform.translation,
          target: pos.0.translation
        });
      }
    }else {
      timer.0.pause();
      timer.0.reset();
    }
}

fn move_fireball(commands: &mut Commands, mut q: Query<(Entity, &Fireball, &mut Transform)>){   
  for (e, f, mut current) in q.iter_mut(){
      current.rotate(Quat::from_rotation_z(0.5));
      let mut translation = &mut current.translation;
      let direction = (f.target - f.origin).normalize();
      translation.x += 7.0 * direction.x;
      translation.y += 7.0 * direction.y;
      if translation.x >= 400.0 || translation.x <= -400.0 || translation.y >= 400.0 || translation.y <= -400.0{
        commands.despawn(e);
      }
      
    }

}
