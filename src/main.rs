use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SPACESHIP_SIZE: Vec3 = Vec3::new(20.0, 50.0, 0.0);
const SPACESHIP_COLOR: Color = Color::rgb(0.0, 0.5, 0.5);

const SPACESHIP_SPEED: f32 = 3000.0;
const SPACESHIP_ANGULAR_SPEED: f32 = 4.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_spaceship)
        .add_system(spaceship_controller)
        .run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_spaceship(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Spaceship)
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(SPACESHIP_SIZE[0], SPACESHIP_SIZE[1]))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(GravityScale(0.0))
        .insert(Sleeping::disabled())
        .insert(ExternalForce {
            force: Vec2 { x: 0.0, y: 0.0 },
            torque: 0.0,
        })
        .insert(Damping {
            linear_damping: 1.0,
            angular_damping: 1.0,
        })
        .insert(SpriteBundle {
            texture: asset_server.load("spaceship.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::from([
                    SPACESHIP_SIZE[0] * 3.0,
                    SPACESHIP_SIZE[1] * 2.0,
                ])),
                ..default()
            },
            ..default()
        });
}

fn spaceship_controller(
    keyboard_input: Res<Input<KeyCode>>,
    mut spaceship: Query<(&mut ExternalForce, &Transform), With<Spaceship>>,
    time_step: Res<FixedTime>,
) {
    let (mut force, trans) = spaceship.single_mut();
    let vector = trans.local_y();

    let mut twist = 0.0;
    let mut thrust = 0.0;

    if keyboard_input.pressed(KeyCode::A) {
        twist += 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        twist -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::W) {
        thrust += 1.0;
    }

    force.torque = twist * SPACESHIP_ANGULAR_SPEED * time_step.period.as_secs_f32();
    force.force = Vec2::new(
        thrust * SPACESHIP_SPEED * time_step.period.as_secs_f32() * vector[0],
        thrust * SPACESHIP_SPEED * time_step.period.as_secs_f32() * vector[1],
    );
}

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct Projectile;
