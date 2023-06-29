use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SPACESHIP_SIZE: Vec3 = Vec3::new(10.0, 20.0, 0.0);

const PROJECTILE_SIZE: Vec3 = Vec3::new(1.0, 5.0, 0.0);
const PROJECTILE_SPEED: f32 = 250.0;

const CANNON_TRANSFORM: Transform = Transform::from_xyz(0.0, 37.0, 0.0);

const SPACESHIP_THRUST: f32 = 1500.0;
const SPACESHIP_TORQUE: f32 = 1.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_event::<ShootEvent>()
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_spaceship)
        .add_system(spaceship_controller)
        .add_system(spawn_projectile)
        .run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_spaceship(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("spaceship.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::from([
                    SPACESHIP_SIZE[0] * 4.0,
                    SPACESHIP_SIZE[1] * 3.0,
                ])),
                ..default()
            },
            ..default()
        })
        .insert(Spaceship)
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule_y(SPACESHIP_SIZE[1], SPACESHIP_SIZE[0]))
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
        .with_children(|parent| {
            parent
                .spawn(Cannon)
                .insert(CannonCooldown {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                })
                .insert(TransformBundle::from(Transform::from(CANNON_TRANSFORM)));
        });
}

fn spawn_projectile(
    mut commands: Commands,
    mut cannon: Query<(&GlobalTransform, &mut CannonCooldown), With<Cannon>>,
    mut shoot_event: EventReader<ShootEvent>,
    time_step: Res<FixedTime>,
    asset_server: Res<AssetServer>,
) {
    let (trans, mut cooldown) = cannon.single_mut();
    if cooldown.timer.finished() {
        for _ in shoot_event.iter() {
            let rot = trans.up();
            commands
                .spawn(SpriteBundle {
                    texture: asset_server.load("laser.png"),
                    sprite: Sprite {
                        custom_size: Some(Vec2::from([
                            SPACESHIP_SIZE[0] * 3.0,
                            SPACESHIP_SIZE[1] * 2.0,
                        ])),
                        ..default()
                    },
                    transform: (*trans).into(),
                    ..default()
                })
                .insert(Projectile)
                .insert(RigidBody::KinematicVelocityBased)
                .insert(Collider::capsule_y(PROJECTILE_SIZE[1], PROJECTILE_SIZE[0]))
                .insert(Velocity {
                    linvel: Vec2 {
                        x: rot[0] * PROJECTILE_SPEED,
                        y: rot[1] * PROJECTILE_SPEED,
                    },
                    angvel: 0.0,
                });
            cooldown.timer.reset();
        }
    }
    cooldown.timer.tick(time_step.period);
}

fn spaceship_controller(
    keyboard_input: Res<Input<KeyCode>>,
    mut spaceship: Query<(&mut ExternalForce, &Transform), With<Spaceship>>,
    time_step: Res<FixedTime>,
    mut shoot_event: EventWriter<ShootEvent>,
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
    if keyboard_input.pressed(KeyCode::Space) {
        shoot_event.send_default();
    }

    force.torque = twist * SPACESHIP_TORQUE * time_step.period.as_secs_f32();
    force.force = Vec2::new(
        thrust * SPACESHIP_THRUST * time_step.period.as_secs_f32() * vector[0],
        thrust * SPACESHIP_THRUST * time_step.period.as_secs_f32() * vector[1],
    );
}

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct Projectile;

#[derive(Default)]
struct ShootEvent;

#[derive(Component)]
struct Cannon;

#[derive(Component)]
struct CannonCooldown {
    timer: Timer,
}
