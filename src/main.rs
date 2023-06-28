use bevy::prelude::*;

const SPACESHIP_SIZE: Vec3 = Vec3::new(20.0, 100.0, 0.0);
const SPACESHIP_COLOR: Color = Color::rgb(0.0, 0.5, 0.5);

const SPACESHIP_SPEED: f32 = 150.0;
const SPACESHIP_ANGULAR_SPEED: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_spaceship)
        .add_system(rotate_spaceship)
        .run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_spaceship(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: SPACESHIP_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: SPACESHIP_COLOR,
                ..default()
            },
            ..default()
        },
        Spaceship,
    ));
}

fn rotate_spaceship(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Spaceship>>,
    time_step: Res<FixedTime>,
) {
    let mut spaceship_transform = query.single_mut();
    let mut twist = 0.0;
    let mut thrust = 0.0;

    if keyboard_input.pressed(KeyCode::A) {
        twist -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        twist += 1.0;
    }
    if keyboard_input.pressed(KeyCode::W) {
        thrust += 1.0;
    }

    spaceship_transform.rotate_z(twist * SPACESHIP_ANGULAR_SPEED * time_step.period.as_secs_f32());

    let spaceship_rot = spaceship_transform.local_y();

    spaceship_transform.translation.x +=
        spaceship_rot[0] * thrust * SPACESHIP_SPEED * time_step.period.as_secs_f32();
    spaceship_transform.translation.y +=
        spaceship_rot[1] * thrust * SPACESHIP_SPEED * time_step.period.as_secs_f32();
}

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct Projectile;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);
