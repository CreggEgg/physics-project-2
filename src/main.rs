use bevy::prelude::*;
use components::Shape;

mod components;

fn main() {
    App::new().add_systems(Startup, setup_world).add_systems(Update, render_shapes).add_plugins(DefaultPlugins).run();
}

fn setup_world(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Shape::Circle(5.0));
}

fn render_shapes(shapes: Query<(Entity,&Shape), Added<Shape>>, mut commands: Commands) {
    for ((entity, shape)) in &shapes {
        let ent = commands.entity(entity);        
    }

}


