use bevy::prelude::*;

fn main() {
  App::build()
  .add_plugins(DefaultPlugins)
  .add_plugin(HelloPlugin)
  .run();
}

//--SYSTEMS--//

fn add_people(commands: &mut Commands){
  commands
    .spawn((Person, Name("Bob".to_string())))
    .spawn((Person, Name("Hugo".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>){
  
  if !timer.0.tick(time.delta_seconds()).just_finished(){
    return;
  }  
  
  for name in query.iter() {
    println!("hello {}!", name.0);
  }
}

//--RESOURCES--//

struct GreetTimer(Timer);

//--COMPONENTS--//

struct Person;

struct Name(String);

//--PLUGINS--//

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
    .add_resource(GreetTimer(Timer::from_seconds(2.0, true)))
    .add_startup_system(add_people.system())
    .add_system(greet_people.system());
  }
}