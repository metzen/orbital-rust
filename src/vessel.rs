use bevy::{
    color::palettes::css::{TEAL, WHITE},
    math::DVec3,
    prelude::*,
};
use big_space::{BigSpace, GridCell, ReferenceFrame};
use rand::{thread_rng, Rng};

use crate::{
    audio::SineAudio,
    camera::{Autoscale, Focusable},
    hud::HudSubject,
    lifetime::Ephemeral,
    physics::{dynamics, NoGravity, RigidBody},
    scene::Planet,
};

pub struct VesselPlugin;

impl Plugin for VesselPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_vessel);
        app.add_systems(
            Update,
            (
                vessel_control,
                vessel_engine_audio,
                animate_engine_particles,
            ),
        );
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
    pub controlled: bool,
    pub throttle: f32, // [0, 1]
    rotate: f32,
    direction_lock: Option<Direction>,
    // # TODO: Maybe initialize to FINE if ecodes.LED_CAPSL in KEYBOARD.leds()
    control_mode: ControlMode,
}

#[derive(Component, Default)]
pub struct EngineParticle;

fn setup_vessel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut assets: ResMut<Assets<SineAudio>>,
    big_space_query: Query<(Entity, &ReferenceFrame<i32>), With<BigSpace>>,
) {
    let (big_space, reference_frame) = big_space_query.single();
    let (grid_cell, translation) = reference_frame.translation_to_grid(DVec3 {
        x: 147.10e9 + 50.0,
        y: Planet::EARTH.radius as f64 + 40.0,
        z: 2.0,
    });
    commands
        .spawn((
            Name::new("Pickle"),
            Transform::from_translation(translation),
            grid_cell,
            Mesh2d(meshes.add(Mesh::from(Capsule2d {
                radius: 8.0,
                half_length: 10.0,
            }))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(TEAL))),
            // Transform::from_xyz(147.10e9 + 500.0, Planet::EARTH.radius, 2.0),
            RigidBody {
                velocity: Vec3 {
                    x: 0.0,
                    y: 30.29e3,
                    z: 0.0,
                },
                mass: 10.0,
                ..default()
            },
            Autoscale,
            Focusable,
            Vessel::default(),
            AudioPlayer(assets.add(SineAudio { frequency: 120.0 })),
            PlaybackSettings {
                spatial: true,
                speed: 0.1,
                ..default()
            },
        ))
        .set_parent(big_space);
    let (grid_cell, translation) = reference_frame.translation_to_grid(DVec3 {
        x: 147.10e9,
        y: Planet::EARTH.radius as f64 + 40.0,
        z: 3.0,
    });
    commands
        .spawn((
            Name::new("FlySafe"),
            Mesh2d(meshes.add(Mesh::from(Capsule2d {
                radius: 10.0,
                half_length: 20.0,
            }))),
            Transform::from_translation(translation),
            grid_cell,
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.78, 0.29, 0.16)))),
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
            HudSubject,
            Vessel::default(),
            AudioPlayer(assets.add(SineAudio { frequency: 150.0 })),
            PlaybackSettings {
                spatial: true,
                speed: 0.1,
                ..default()
            },
        ))
        .set_parent(big_space)
        .with_children(|vessel| {
            vessel.spawn((
                Name::new("Hot dog bun 1"),
                Mesh2d(meshes.add(Mesh::from(Capsule2d {
                    radius: 10.0,
                    half_length: 20.0,
                }))),
                Transform::from_xyz(-10.0, 0.0, -1.0),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.9, 0.58, 0.27)))),
            ));
            vessel.spawn((
                Name::new("Hot dog bun 2"),
                Mesh2d(meshes.add(Mesh::from(Capsule2d {
                    radius: 10.0,
                    half_length: 20.0,
                }))),
                Transform::from_xyz(10.0, 0.0, -1.0),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.9, 0.58, 0.27)))),
            ));
        });
}

fn vessel_control(mut query: Query<&mut Vessel>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for mut vessel in query.iter_mut() {
        if !vessel.controlled {
            continue;
        }

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

fn vessel_engine_audio(query: Query<(&Vessel, Option<&SpatialAudioSink>)>) {
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
fn vessel_systems(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut RigidBody, &Vessel, &GridCell<i32>)>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    big_space_query: Query<Entity, With<BigSpace>>,
) {
    for (mut transform, mut rigidbody, vessel, grid_cell) in query.iter_mut() {
        if vessel.rotate != 0.0 {
            transform.rotate_z(vessel.rotate * time.delta_secs());
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
                    Mesh2d(meshes.add(Mesh::from(Cuboid::new(2.5, 2.5, 1.0)))),
                    Transform::from_translation(
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
                    MeshMaterial2d(materials.add(ColorMaterial {
                        color: Color::srgba(10.0, 6.0, 1.0, 1.0),
                        alpha_mode: bevy::sprite::AlphaMode2d::Blend,
                        texture: Option::None,
                    })),
                    *grid_cell,
                    // TODO: Fix this velocity
                    RigidBody {
                        velocity: rigidbody.velocity
                            + ((transform.rotation
                                * Vec3 {
                                    x: thread_rng().gen_range(-0.2..0.2),
                                    y: -1.0,
                                    z: 0.0,
                                })
                                * (force_magnitude / (rigidbody.mass / 1000.0))
                                * time.delta_secs()),
                        mass: 1.0,
                        ..default()
                    },
                    NoGravity,
                    Autoscale,
                    Ephemeral { ttl: 60 * 5 },
                    EngineParticle,
                ))
                .set_parent(big_space_query.single());
        }
    }
}

fn animate_engine_particles(
    mut query: Query<(&mut Transform, &MeshMaterial2d<ColorMaterial>), With<EngineParticle>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (mut transform, color_material_handle) in query.iter_mut() {
        transform.scale *= 1.005;
        let material = color_materials.get_mut(color_material_handle).unwrap();
        if material.color.luminance() > 1.0 {
            material.color.mix_assign(material.color.darker(0.5), 0.1);
        } else {
            material
                .color
                .mix_assign(Color::srgba(1.0, 1.0, 1.0, material.color.alpha()), 0.3);
        }
    }
}
