use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;
use bevy_turborand::prelude::*;
use std::f32::consts::PI;
use std::ops::Mul;
use std::time::Duration;

const SPACESHIP_SIZE: Vec3 = Vec3::new(10.0, 20.0, 0.0);
const SPACESHIP_THRUST: f32 = 2.0;
const SPACESHIP_TORQUE: f32 = 0.05;

const PROJECTILE_SIZE: Vec3 = Vec3::new(1.0, 5.0, 0.0);
const PROJECTILE_SPEED: f32 = 900.0;

const CANNON_TRANSFORM: Transform = Transform::from_xyz(0.0, 37.0, 0.0);

const MAX_ASTEROID_RADIUS: f32 = 30.0;
const MIN_ASTEROID_RADIUS: f32 = 10.0;
const MAX_ASTEROID_SPEED: f32 = 80.0;
const MIN_ASTEROID_SPEED: f32 = 40.0;
const ASTEROID_SPAWN_RADIUS: f32 = 2000.0;

const ASTEROID_PERIOD: f32 = 12.0;

const PLANET_GRAVITY: f32 = 600.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                //mode: WindowMode::Fullscreen,
                resolution: WindowResolution::new(1500.0, 1500.0).with_scale_factor_override(1.0),
                //title: String::from("Asteroid Inc."),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .insert_resource(AsteroidCooldown {
            timer: Timer::from_seconds(ASTEROID_PERIOD, TimerMode::Repeating),
        })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RngPlugin::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_event::<ShootEvent>()
        .add_event::<AsteroidEvent>()
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_spaceship)
        .add_startup_system(spawn_planet)
        .add_system(spaceship_controller)
        .add_system(spawn_projectile)
        .add_system(asteroid_shower)
        .add_system(spawn_asteroid)
        .add_system(planet_gravity)
        .add_system(asteroid_collision_handler)
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
            transform: Transform::from_xyz(-375.0, 0.0, 0.0),
            ..default()
        })
        .insert(Spaceship)
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule_y(SPACESHIP_SIZE[1], SPACESHIP_SIZE[0]))
        .insert(Sleeping::disabled())
        .insert(ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        })
        .insert(ExternalImpulse {
            impulse: Vec2::ZERO,
            torque_impulse: 0.0,
        })
        .insert(Velocity {
            linvel: Vec2 { x: 0.0, y: 200.0 },
            angvel: 0.0,
        })
        .insert(Damping {
            linear_damping: 0.005,
            angular_damping: 3.0,
        })
        .insert(Orbiting)
        .with_children(|parent| {
            parent
                .spawn(Cannon)
                .insert(CannonCooldown {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                })
                .insert(TransformBundle::from(CANNON_TRANSFORM));
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
                .insert(RigidBody::Dynamic)
                .insert(Collider::capsule_y(PROJECTILE_SIZE[1], PROJECTILE_SIZE[0]))
                .insert(AdditionalMassProperties::Mass(1000.0))
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

fn spawn_asteroid(
    mut commands: Commands,
    mut asteroid_event: EventReader<AsteroidEvent>,
    asset_server: Res<AssetServer>,
) {
    for asteroid in asteroid_event.iter() {
        let asteroid_trans = Transform::from_xyz(asteroid.0.x, asteroid.0.y, asteroid.0.z)
            .looking_at(asteroid.1, Vec3::Z);
        let force_vec = asteroid_trans.forward();
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("asteroid.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::from([asteroid.3 * 2.0, asteroid.3 * 2.0])),
                    ..default()
                },
                transform: asteroid_trans,
                ..default()
            })
            .insert(Asteroid)
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(asteroid.3))
            .insert(Velocity {
                linvel: force_vec.truncate().mul(asteroid.2),
                angvel: 0.0,
            })
            .insert(ExternalForce::default())
            .insert(Orbiting)
            .insert(ActiveEvents::COLLISION_EVENTS)  
            .insert(ActiveEvents::CONTACT_FORCE_EVENTS);
    }
}

fn spawn_planet(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("planet.png"),
            sprite: Sprite {
                custom_size: Some(Vec2 { x: 410.0, y: 410.0 }),
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(200.0))
        .insert(GravityWell {
            intensity: PLANET_GRAVITY,
            max_radius: 600.0,
        });
}

fn asteroid_shower(
    mut asteroid_event: EventWriter<AsteroidEvent>,
    mut asteroid_timer: ResMut<AsteroidCooldown>,
    mut rng: ResMut<GlobalRng>,
) {
    if asteroid_timer.timer.finished() {
        let angle = rng.f32() * PI * 2.0;
        let origin = Vec3::new(angle.cos(), angle.sin() / 2.0, 0.0).mul(ASTEROID_SPAWN_RADIUS);
        let target = Vec3::new(rng.f32() * 10.0 - 5.0, rng.f32() * 10.0 - 5.0, 0.0);
        let speed = rng.f32() * (MAX_ASTEROID_SPEED - MIN_ASTEROID_SPEED) + MIN_ASTEROID_SPEED;
        let radius = rng.f32() * (MAX_ASTEROID_RADIUS - MIN_ASTEROID_RADIUS) + MIN_ASTEROID_RADIUS;
        asteroid_event.send(AsteroidEvent(origin, target, speed, radius));
    }
    asteroid_timer
        .timer
        .tick(Duration::from_secs_f32(rng.f32() % 0.05));
}

fn spaceship_controller(
    keyboard_input: Res<Input<KeyCode>>,
    mut spaceship: Query<(&mut ExternalImpulse, &Transform), With<Spaceship>>,
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

    force.torque_impulse = twist * SPACESHIP_TORQUE * time_step.period.as_secs_f32();
    force.impulse = Vec2::new(
        thrust * SPACESHIP_THRUST * time_step.period.as_secs_f32() * vector[0],
        thrust * SPACESHIP_THRUST * time_step.period.as_secs_f32() * vector[1],
    );
}

fn planet_gravity(
    well: Query<(&GlobalTransform, &GravityWell)>,
    mut body: Query<(&GlobalTransform, &mut ExternalForce), With<Orbiting>>,
    timer: Res<FixedTime>,
) {
    for (centre, well) in well.iter() {
        for (body_glob, mut force) in body.iter_mut() {
            let distance = (body_glob.translation() - centre.translation()).length();

            if distance > 0.0 && distance < well.max_radius {
                let pull = (well.intensity / distance * distance) * timer.period.as_secs_f32();

                force.force = body_glob.translation().truncate().normalize().mul(-pull);
            }
        }
    }
}

fn asteroid_collision_handler(mut contact_event: EventReader<ContactForceEvent>,
                              asteroids: Query<(Entity, &Asteroid), With<Asteroid>>,
                              projectiles: Query<(Entity, &Projectile), With<Projectile>>,
                              mut commands: Commands,
                              ) {
    for collision in contact_event.iter() {
        for (asteroid, _ ) in asteroids.iter() {
            for (projectile, _ ) in projectiles.iter() {
                let ContactForceEvent{
                    collider1: h1,
                    collider2: h2,
                    total_force: _,
                    total_force_magnitude: _,
                    max_force_direction: _,
                    max_force_magnitude: _,
                } = collision; 
                    if  (&projectile == h1 && &asteroid == h2) || (&asteroid == h1 && &projectile == h2) {
                        commands.entity(projectile).despawn();
                    
                 }
            }
        }
    }         
}

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct Cannon;

#[derive(Component)]
struct GravityWell {
    intensity: f32,
    max_radius: f32,
}

#[derive(Component)]
struct Orbiting;

#[derive(Component)]
struct CannonCooldown {
    timer: Timer,
}

#[derive(Resource)]
struct AsteroidCooldown {
    timer: Timer,
}

#[derive(Default)]
struct ShootEvent;

#[derive(Default)]
struct AsteroidEvent(Vec3, Vec3, f32, f32);
