use std::f32::consts::PI;

use bevy::{
    gizmos,
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    utils::warn,
    window::PrimaryWindow,
};
use components::{DynamicObject, Force, Shape, StaticObject};
use ops::{atan2, cos, sin};

mod components;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug)]
enum SimState {
    Waiting,
    Running,
}

fn main() {
    App::new()
        .add_systems(Startup, setup_world)
        .add_systems(Update, render_shapes)
        .add_systems(Update, wait.run_if(in_state(SimState::Waiting)))
        .add_systems(
            Update,
            (spawn_ball, update_cursor_position).run_if(in_state(SimState::Running)),
        )
        .add_systems(
            FixedUpdate,
            (
                (
                    empty_forces,
                    (apply_gravity),
                    (normal_force, apply_forces).chain(),
                )
                    .chain(),
                apply_velocity,
            )
                .chain()
                .run_if(in_state(SimState::Running)),
        )
        .add_plugins(DefaultPlugins)
        .insert_state(SimState::Waiting)
        .init_resource::<CursorCoords>()
        .run();
}

fn setup_world(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
    commands.spawn((
        Shape::Rect(10000.0, 50.),
        Transform::from_xyz(0., -500.0, 0.),
        StaticObject {},
    ));
    commands.spawn((Shape::Circle(50.0), DynamicObject::new(5.0)));
}

fn render_shapes(
    shapes: Query<(Entity, &Shape), Added<Shape>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for ((entity, shape)) in &shapes {
        let mesh = meshes.add(match shape {
            Shape::Circle(radius) => Into::<Mesh>::into(Circle::new(*radius)),
            Shape::Rect(width, height) => Rectangle::new(*width, *height).into(),
        });
        let mut entity = commands.entity(entity);
        entity.insert((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0., 0.))),
        ));
    }
}
fn apply_velocity(
    mut dynamic_objects: Query<(&mut Transform, &DynamicObject)>,
    time: Res<Time<Fixed>>,
) {
    for (mut transform, dynamic_object) in &mut dynamic_objects {
        transform.translation +=
            dynamic_object.velocity.extend(0.) * time.delta_secs() * Vec3::splat(2.0);
    }
}

fn apply_forces(mut dynamic_objects: Query<&mut DynamicObject>) {
    for mut dynamic_object in &mut dynamic_objects {
        let mut additional_velocity = Vec2::ZERO;
        for force in &dynamic_object.forces {
            let magnitude = force.magnitude / dynamic_object.mass;
            let acceleration_vector =
                Vec2::new(magnitude * cos(force.angle), magnitude * sin(force.angle));
            additional_velocity += acceleration_vector;
        }
        let last_vel = dynamic_object.velocity.length();
        dynamic_object.velocity += additional_velocity;
        if dynamic_object.velocity.length() - last_vel > 0.0 {
            // eprintln!("ACCELERATING");
        }
        if dynamic_object.velocity.length() - last_vel < 0.0 {
            eprintln!("DECELERATING");
        }
    }
}

fn empty_forces(mut dynamic_objects: Query<&mut DynamicObject>) {
    for mut dynamic_object in &mut dynamic_objects {
        let _ = dynamic_object.forces.drain(..);
    }
}
fn apply_gravity(mut dynamic_objects: Query<&mut DynamicObject>) {
    for mut dynamic_object in &mut dynamic_objects {
        let mass = dynamic_object.mass;
        dynamic_object
            .forces
            .push(Force::from_x_and_y(0.0, -9.8 * mass));
    }
}
fn normal_force(
    mut objects: Query<(Option<&mut DynamicObject>, &mut Transform, &Shape)>,
    mut gizmos: Gizmos,
) {
    let mut objects: Vec<_> = objects.iter_mut().collect();
    for i in 0..objects.len() {
        if let None = objects[i].0 {
            continue;
        }
        'a: for j in 0..objects.len() {
            if i == j {
                continue 'a;
            }
            let main_translation = objects[i].1.translation.xy();
            let other_translation = objects[j].1.translation.xy();

            let intersects =
                objects[i]
                    .2
                    .intersects(main_translation, objects[j].2, other_translation);
            if intersects {
                let other_closest_point = objects[j]
                    .2
                    .closest_point(other_translation, objects[i].2, main_translation)
                    .clone();
                let delta = (main_translation - other_closest_point).normalize_or_zero();
                let delta_angle = delta.to_angle();
                // objects[i].1.translation += (Vec2::splat(0.1) * delta).extend(0.);

                if let Some(dynamic_object) = &mut objects[i].0 {
                    let opposing_force_magnitude = {
                        let mut magnitude = 0.;
                        for force in &dynamic_object.forces {
                            let adjusted_angle = force.angle - delta_angle;
                            let mag = cos(adjusted_angle) * force.magnitude;
                            magnitude += mag;
                        }
                        magnitude
                    };
                    let opposing_velocity_magnitude = {
                        let velocity_magnitude = dynamic_object.velocity.length();
                        let velocity_angle =
                            atan2(dynamic_object.velocity.y, dynamic_object.velocity.x);
                        let mag = cos(velocity_angle - delta_angle) * velocity_magnitude;
                        mag
                    };

                    // gizmos.arrow_2d(
                    //     main_translation,
                    //     main_translation
                    //         + Vec2::new(cos(delta_angle), sin(delta_angle))
                    //             * -Vec2::splat(50. * opposing_force_magnitude),
                    //     Color::srgb(0., 1., 0.),
                    // );

                    // dynamic_object.velocity = Vec2::ZERO;
                    eprintln!("{}", opposing_force_magnitude);

                    let mass = dynamic_object.mass;
                    dynamic_object.forces.push(Force {
                        magnitude: (-opposing_force_magnitude
                            + (-1.5 * opposing_velocity_magnitude * mass)),
                        angle: delta_angle,
                    });
                    // let opposing_force_magnitude = {
                    //     let mut magnitude = 0.;
                    //     for force in &dynamic_object.forces {
                    //         let adjusted_angle = force.angle - delta_angle;
                    //         let mag = cos(adjusted_angle) * force.magnitude;
                    //         magnitude += mag;
                    //     }
                    //     magnitude
                    // };
                    // eprintln!("after: {}", opposing_force_magnitude);
                    let normal_force = opposing_force_magnitude.abs();

                    let perpendicular_angle = /* 2. *  */delta_angle + PI / 2.0;

                    let velocity_along_tangent_sign = {
                        // let velocity_magnitude = dynamic_object.velocity.length_squared();
                        let velocity_angle =
                            atan2(dynamic_object.velocity.y, dynamic_object.velocity.x);
                        cos(velocity_angle)
                    };
                    dynamic_object.forces.push(Force {
                        magnitude: (if velocity_along_tangent_sign > 0.5 {
                            velocity_along_tangent_sign
                        } else {
                            0.
                        }) * normal_force
                            * 0.15,
                        angle: perpendicular_angle,
                    });
                    // gizmos.arrow_2d(
                    //     main_translation,
                    //     main_translation
                    //         + Vec2::new(cos(perpendicular_angle), sin(perpendicular_angle))
                    //             * -Vec2::splat(50. * velocity_along_tangent_sign),
                    //     Color::srgb(0., 1., 0.),
                    // );
                };
            }
            // let intersects = {
            //     let position = objects[i].1.translation.xy();
            //     match objects[i].2 {
            //         Shape::Circle(radius) => BoundingCircle::new(position, *radius),
            //         Shape::Rect(width, height) => {
            //             Aabb2d::new(position, Vec2::new(width / 2.0, height / 2.0))
            //         }
            //     }
            // }
            // .intersects({
            //     let position = objects[j].1.translation.xy();
            //     match objects[j].2 {
            //         Shape::Circle(radius) => &BoundingCircle::new(position, *radius),
            //         Shape::Rect(width, height) => {
            //             &Aabb2d::new(position, Vec2::new(width / 2.0, height / 2.0))
            //         }
            //     }
            // });
        }
    }
}

fn wait(input: Res<ButtonInput<MouseButton>>, mut next_state: ResMut<NextState<SimState>>) {
    if input.just_pressed(MouseButton::Left) {
        next_state.set(SimState::Running);
    }
}
#[derive(Resource, Default)]
struct CursorCoords(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn update_cursor_position(
    mut coords: ResMut<CursorCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        coords.0 = world_position;
    }
}
fn spawn_ball(
    input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorCoords>,
    mut commands: Commands,
) {
    if input.just_pressed(MouseButton::Left) {
        commands.spawn((
            Shape::Circle(50.0),
            DynamicObject::new(5.0),
            Transform::from_xyz(cursor_pos.0.x, cursor_pos.0.y, 0.),
        ));
    }
}
