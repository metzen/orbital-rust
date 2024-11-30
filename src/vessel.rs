use bevy::{
    color::palettes::css::{TEAL, WHITE, YELLOW},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use big_space::{BigSpace, GridCell};
use rand::{thread_rng, Rng};

use crate::{
    audio::SineAudio,
    camera::{Autoscale, Focusable},
    lifetime::Ephemeral,
    physics::{dynamics, NoGravity, RigidBody},
    scene::Planet,
};

pub struct VesselPlugin;

impl Plugin for VesselPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_vessel);
        app.add_systems(Update, (vessel_control, vessel_engine_audio));
        app.add_systems(FixedPreUpdate, vessel_systems.before(dynamics));
    }
}

#[derive(Default, PartialEq)]
enum ControlMode {
    #[default]
    Normal,
    Fine,
}

enum Direction {
    Prograde,
    Retrograde,
    Radial,
    AntiRadial,
}

#[derive(Component, Default)]
pub struct Vessel {
    controlled: bool,
    pub throttle: f32, // [0, 1]
    rotate: f32,
    direction_lock: Option<Direction>,
    // # TODO: Maybe initialize to FINE if ecodes.LED_CAPSL in KEYBOARD.leds()
    control_mode: ControlMode,
}

pub fn setup_vessel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut assets: ResMut<Assets<SineAudio>>,
    big_space_query: Query<Entity, With<BigSpace>>,
) {
    let big_space = big_space_query.single();
    commands
        .spawn((
            Name::new("FlySafe"),
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(Mesh::from(Capsule2d {
                        radius: 10.0,
                        half_length: 20.0,
                    }))
                    .into(),
                transform: Transform::from_xyz(147.10e9, Planet::EARTH.radius, 1.0),
                material: materials.add(ColorMaterial::from(Color::srgb(0.78, 0.29, 0.16))),
                ..default()
            },
            RigidBody {
                velocity: Vec3 {
                    x: 0.0,
                    y: 30.29e3,
                    z: 0.0,
                },
                mass: 100.0,
                ..default()
            },
            Autoscale,
            Focusable,
            Vessel::default(),
            GridCell::<i32>::default(),
            AudioSourceBundle {
                source: assets.add(SineAudio { frequency: 150.0 }),
                ..default()
            },
        ))
        .set_parent(big_space)
        .with_children(|vessel| {
            vessel.spawn((
                Name::new("Hot dog bun"),
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(Mesh::from(Capsule2d {
                            radius: 15.0,
                            half_length: 20.0,
                        }))
                        .into(),
                    transform: Transform::from_xyz(0.0, 0.0, -1.0),
                    material: materials.add(ColorMaterial::from(Color::srgb(0.9, 0.58, 0.27))),
                    ..default()
                },
                Autoscale,
            ));
        });
}

pub fn vessel_control(mut query: Query<(&mut Vessel)>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for (mut vessel) in query.iter_mut() {
        // if !vessel.controlled {
        //     continue;
        // }

        if keyboard_input.just_pressed(KeyCode::CapsLock) {
            vessel.control_mode = match vessel.control_mode {
                ControlMode::Normal => ControlMode::Fine,
                ControlMode::Fine => ControlMode::Normal,
            }
        }

        if keyboard_input.pressed(KeyCode::KeyZ) {
            vessel.throttle = 1.0
        }

        if keyboard_input.pressed(KeyCode::KeyX) {
            vessel.throttle = 0.0
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            let change = match vessel.control_mode {
                ControlMode::Normal => 0.01,
                ControlMode::Fine => 0.0005,
            };
            vessel.throttle = (vessel.throttle + change).clamp(0.0, 1.0);
            info!("Throttle: {}", vessel.throttle);
        }
        if keyboard_input.pressed(KeyCode::ControlLeft) {
            let change = match vessel.control_mode {
                ControlMode::Normal => -0.01,
                ControlMode::Fine => -0.0005,
            };
            vessel.throttle = (vessel.throttle + change).clamp(0.0, 1.0);
            info!("Throttle: {}", vessel.throttle);
        }

        // TODO: Do this as angular torque instead of setting rotation directly.
        vessel.rotate = 0.0;
        if keyboard_input.pressed(KeyCode::KeyA) {
            // # TODO: factor = 0.5 if ecodes.LED_CAPSL in KEYBOARD.leds() else 2
            let angle: f32 = if vessel.control_mode == ControlMode::Normal {
                200.0
            } else {
                10.0
            };
            vessel.direction_lock = Option::None;
            vessel.rotate = angle.to_radians();

            // transform.rotation = (transform.rotation + PI * 2.0 / 360.0 * factor) % (2 * PI)
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            // # TODO: factor = 0.5 if ecodes.LED_CAPSL in KEYBOARD.leds() else 2
            let angle: f32 = if vessel.control_mode == ControlMode::Normal {
                -200.0
            } else {
                -10.0
            };
            vessel.direction_lock = Option::None;
            vessel.rotate = angle.to_radians();

            // transform.rotation = (transform.rotation + PI * 2.0 / 360.0 * factor) % (2 * PI)
        }
        //     if action_states[VesselAction.ROTATE_CLOCKWISE] == ActionState.ACTIVE:
        //         # TODO: factor = 0.5 if ecodes.LED_CAPSL in KEYBOARD.leds() else 2
        //         factor = 2 if vessel.control_mode == ControlMode.NORMAL else 1
        //         vessel.direction_lock = None
        //         transform.rotation = (transform.rotation - math.pi * 2 / 360 * factor) % (
        //             2 * math.pi
        //         )

        if keyboard_input.pressed(KeyCode::KeyP) {
            vessel.direction_lock = Some(Direction::Prograde);
        }
        if keyboard_input.pressed(KeyCode::KeyR) {
            vessel.direction_lock = Some(Direction::Retrograde);
        }
        if keyboard_input.pressed(KeyCode::KeyO) {
            vessel.direction_lock = Some(Direction::Radial);
        }
        if keyboard_input.pressed(KeyCode::KeyI) {
            vessel.direction_lock = Some(Direction::AntiRadial);
        }
    }
}

pub fn vessel_engine_audio(query: Query<(&Vessel, Option<&AudioSink>)>) {
    for (vessel, audiosink) in &query {
        if audiosink.is_some() {
            let sink = audiosink.unwrap();
            sink.set_speed(if vessel.throttle < 0.1 {
                0.1
            } else {
                vessel.throttle
            });
        }
    }
}

// Applies effects of active vessel controls.
pub fn vessel_systems(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut RigidBody,
        &Vessel,
        &GridCell<i32>,
        Option<&AudioSink>,
    )>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    big_space_query: Query<Entity, With<BigSpace>>,
) {
    for (entity, mut transform, mut rigidbody, vessel, grid_cell, audiosink) in query.iter_mut() {
        if vessel.rotate != 0.0 {
            transform.rotate_z(vessel.rotate * time.delta_seconds());
        }
        if vessel.throttle > 0.0 {
            // info!("Throttle: {}", vessel.throttle);
            let acceleration = 9.8 * 3.0; // m/s**2
            let force_magnitude = rigidbody.mass * acceleration * vessel.throttle;
            // direction = Vector2D(
            //     math.cos(transform.rotation), math.sin(transform.rotation)
            // )

            // info!("{}, {}", transform.rotation, transform.rotation.xyz());
            let engine_force = transform.rotation * Vec3::Y * force_magnitude;

            // rigidbody.force += Vec3 {
            //     x: 0.0,
            //     y: force_magnitude,
            //     z: 0.0,
            // };
            // info!("engforce {}", engine_force);
            rigidbody.force += engine_force;
            // TODO: Refactor to a better particle system.
            // TODO: Do this with an api that clones from entity.
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(Circle::new(2.5))).into(),
                        transform: Transform::from_translation(
                            transform.translation
                            + Vec3 {
                                z: -1.0,
                                ..default()
                            }
                            // Emit from rear of vessel.
                            + transform.rotation * -Vec3::Y * 25.0,
                        ),
                        // transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        // material: materials.add(ColorMaterial::from(Color::srgb(0.96, 0.79, 0.11))),
                        material: materials.add(ColorMaterial::from(Color::from(WHITE))),
                        ..default()
                    },
                    *grid_cell,
                    // TODO: Fix this velocity
                    RigidBody {
                        velocity: rigidbody.velocity
                            + ((transform.rotation
                                * Vec3 {
                                    x: thread_rng().gen_range(-0.15..0.15),
                                    y: -1.0,
                                    z: 0.0,
                                })
                                * (force_magnitude / (rigidbody.mass / 1000.0))
                                * time.delta_seconds()),
                        mass: 1.0,
                        ..default()
                    },
                    NoGravity,
                    Autoscale,
                    Ephemeral { ttl: 60 * 5 },
                ))
                .set_parent(big_space_query.single());
        }
    }
}
