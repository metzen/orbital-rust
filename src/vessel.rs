use std::f32::consts::PI;
use std::time::Duration;

use bevy::camera::primitives::Aabb;
use bevy::color::palettes::css::{BROWN, GREEN, RED, TEAL};
use bevy::color::palettes::tailwind::BLUE_600;
use bevy::ecs::entity_disabling::Disabled;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::{Read, Write};
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::prelude::*;
use rand::{Rng, rng};

use crate::audio::SineAudio;
use crate::camera::{Autoscale, Focusable};
use crate::hud::HudSubject;
use crate::lifetime::{Clock, Ephemeral, ExpirationAction};
use crate::physics::{
    Collider, Drag, NoGravity, PhysicsMaterial, RigidBody, SPEED_OF_LIGHT, SatelliteOf,
};
use crate::scene::Planet;
use crate::timewarp::TimeWarp;

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
                fire_photon.run_if(action_just_pressed(VesselAction::FirePhoton)),
            ),
        );
        // Must run after final changes have been applied to vessel translation in FixedUpdate.
        app.add_systems(FixedPostUpdate, vessel_systems);
        app.add_plugins(InputManagerPlugin::<VesselAction>::default());
        app.init_resource::<ActionState<VesselAction>>();
        app.insert_resource(EngineParticleSpawnTimer(Timer::new(
            Duration::from_millis(50),
            TimerMode::Repeating,
        )));
        app.insert_resource(VesselAction::default_input_map());
    }
}

#[derive(Default, PartialEq, Reflect)]
enum ControlMode {
    #[default]
    Normal,
    Fine,
}

#[derive(Reflect)]
pub enum Direction {
    Prograde,
    Retrograde,
    Radial,
    AntiRadial,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum VesselAction {
    #[actionlike(Axis)]
    Rotate,
    SasModeAntiRadial,
    SasModePrograde,
    SasModeRadial,
    SasModeRetrograde,
    SasToggle,
    #[actionlike(Axis)]
    Throttle,
    ThrottleIncrease,
    ThrottleDecrease,
    ThrottleOpen,
    ThrottleClose,
    TogglePrecisionControls,
    FirePhoton,
}

impl VesselAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with_axis(
                Self::Rotate,
                GamepadControlAxis::LEFT_X.with_deadzone_symmetric(0.3),
            )
            .with_axis(Self::Rotate, VirtualAxis::ad())
            .with_axis(
                Self::Throttle,
                VirtualAxis::new(GamepadButton::LeftTrigger2, GamepadButton::RightTrigger2),
            )
            .with_axis(
                Self::Throttle,
                VirtualAxis::new(KeyCode::ControlLeft, KeyCode::ShiftLeft),
            )
            .with(Self::ThrottleOpen, KeyCode::KeyZ)
            .with(Self::ThrottleOpen, GamepadButton::RightTrigger)
            .with(Self::ThrottleClose, KeyCode::KeyX)
            .with(Self::ThrottleClose, GamepadButton::LeftTrigger)
            .with(Self::TogglePrecisionControls, KeyCode::CapsLock)
            .with(Self::FirePhoton, KeyCode::Space)
            .with(Self::SasModePrograde, KeyCode::KeyP)
            .with(
                Self::SasModePrograde,
                ButtonlikeChord::new([GamepadButton::North, GamepadButton::DPadUp]),
            )
            .with(Self::SasModeRetrograde, KeyCode::KeyR)
            .with(
                Self::SasModeRetrograde,
                ButtonlikeChord::new([GamepadButton::North, GamepadButton::DPadDown]),
            )
            .with(Self::SasModeRadial, KeyCode::KeyO)
            .with(
                Self::SasModeRadial,
                ButtonlikeChord::new([GamepadButton::North, GamepadButton::DPadLeft]),
            )
            .with(Self::SasModeAntiRadial, KeyCode::KeyI)
            .with(
                Self::SasModeAntiRadial,
                ButtonlikeChord::new([GamepadButton::North, GamepadButton::DPadRight]),
            )
            .with(Self::SasToggle, KeyCode::KeyT)
            .with(Self::SasToggle, GamepadButton::North)
    }
}

#[derive(Component, Default, Reflect)]
pub struct Vessel {
    pub controlled: bool,
    pub throttle: f32, // [0, 1]
    pub engine_translation: Vec3,
    rotate: f32,
    pub direction_lock: Option<Direction>,
    // # TODO: Maybe initialize to FINE if ecodes.LED_CAPSL in KEYBOARD.leds()
    control_mode: ControlMode,
    pub sas_enabled: bool,
}

#[derive(Component, Default)]
pub struct EngineParticle;

#[derive(Resource, Default)]
struct EngineParticleSpawnTimer(Timer);

pub trait VesselCommands {
    /// Spawn a [`Vessel`].
    fn spawn_vessel(&mut self, bundle: impl Bundle) -> EntityCommands<'_>;
}

impl VesselCommands for Commands<'_, '_> {
    fn spawn_vessel(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let mut commands = self.spawn(bundle);
        commands.insert((
            Autoscale::new(2.0),
            Collider,
            Drag,
            Focusable,
            Pickable::default(),
            PlaybackSettings::ONCE.paused().with_spatial(true),
        ));
        commands
    }
}

fn setup_vessel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut assets: ResMut<Assets<SineAudio>>,
    big_space: Single<(Entity, &Grid), With<BigSpace>>,
) {
    let (big_space, grid) = big_space.into_inner();
    let (grid_cell, translation) = grid.translation_to_grid(DVec3 {
        x: 147.10e9,
        y: Planet::EARTH.radius as f64 + 40.0,
        z: 4.0,
    });
    commands.spawn_vessel((
        Name::new("Falcon 9"),
        Transform::from_translation(translation + Vec3::X * 150.0),
        grid_cell,
        Mesh2d(meshes.add(Mesh::from(Capsule2d::new(1.85, 70.0)))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(TEAL))),
        // Transform::from_xyz(147.10e9 + 500.0, Planet::EARTH.radius, 2.0),
        RigidBody {
            velocity: Vec3 {
                x: 0.0,
                y: 30.29e3,
                z: 0.0,
            },
            mass: 549_000.0,
            ..default()
        },
        PhysicsMaterial { restituion: 0.5 },
        HudSubject,
        Vessel {
            engine_translation: -Vec3::Y * (1.85 + 35.0),
            controlled: true,
            sas_enabled: true,
            ..default()
        },
        AudioPlayer(assets.add(SineAudio::new(120.0))),
        ChildOf(big_space),
    ));
    commands.spawn_vessel((
        Name::new("Pizza"),
        Mesh2d(meshes.add(Mesh::from(Triangle2d::new(
            Vec2::new(0.0, 20.0),
            Vec2::new(-13.0, -15.0),
            Vec2::new(13.0, -15.0),
        )))),
        Transform::from_translation(translation + Vec3::X * 75.0),
        grid_cell,
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgba(1.0, 0.8, 0.54, 1.0)))),
        RigidBody {
            velocity: Vec3 {
                x: 0.0,
                y: 30.29e3,
                z: 0.0,
            },
            mass: 100_000.0,
            ..default()
        },
        Vessel {
            engine_translation: -Vec3::Y * 15.0,
            sas_enabled: true,
            ..default()
        },
        AudioPlayer(assets.add(SineAudio::new(150.0))),
        ChildOf(big_space),
        children![
            (
                Name::new("pepperoni"),
                Transform::from_xyz(-2.0, -2.0, 1.0),
                Mesh2d(meshes.add(Mesh::from(Circle::new(2.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
            ),
            (
                Name::new("pepperoni"),
                Transform::from_xyz(3.0, -8.0, 1.0),
                Mesh2d(meshes.add(Mesh::from(Circle::new(2.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
            ),
            (
                Name::new("pepperoni"),
                Transform::from_xyz(1.0, 5.0, 1.0),
                Mesh2d(meshes.add(Mesh::from(Circle::new(2.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
            ),
            (
                Name::new("crust"),
                Transform::from_xyz(0.0, -15.0, 1.0).with_rotation(Quat::from_rotation_z(PI / 2.0)),
                Mesh2d(meshes.add(Mesh::from(Capsule2d::new(3.0, 20.0)))),
                MeshMaterial2d(
                    materials.add(ColorMaterial::from(Color::srgba(0.96, 0.69, 0.24, 1.0))),
                ),
            )
        ],
    ));
    commands.spawn_vessel((
        Name::new("Hotdog"),
        Transform::from_translation(translation),
        grid_cell,
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::new(20.0, 30.0, 0.0),
        },
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.78, 0.29, 0.16)))),
        RigidBody {
            velocity: Vec3 {
                x: 0.0,
                y: 30.29e3,
                z: 0.0,
            },
            mass: 200_000.0,
            ..default()
        },
        Vessel {
            engine_translation: -Vec3::Y * 30.0,
            sas_enabled: true,
            ..default()
        },
        AudioPlayer(assets.add(SineAudio::new(150.0))),
        ChildOf(big_space),
        children![
            (
                Name::new("dog"),
                Mesh2d(meshes.add(Mesh::from(Capsule2d::new(10.0, 40.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.78, 0.29, 0.16)))),
            ),
            (
                Name::new("Hot dog bun 1"),
                Transform::from_xyz(-10.0, 0.0, -1.0),
                Mesh2d(meshes.add(Mesh::from(Capsule2d::new(10.0, 40.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.9, 0.58, 0.27)))),
            ),
            (
                Name::new("Hot dog bun 2"),
                Transform::from_xyz(10.0, 0.0, -1.0),
                Mesh2d(meshes.add(Mesh::from(Capsule2d::new(10.0, 40.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.9, 0.58, 0.27)))),
            ),
        ],
    ));
    commands.spawn_vessel((
        Name::new("Christmas Tree"),
        Mesh2d(meshes.add(Mesh::from(Triangle2d::new(
            Vec2::new(0.0, 20.0),
            Vec2::new(-13.0, -15.0),
            Vec2::new(13.0, -15.0),
        )))),
        Transform::from_translation(translation + Vec3::X * 275.0),
        grid_cell,
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(GREEN)))),
        RigidBody {
            velocity: Vec3 {
                x: 0.0,
                y: 30.29e3,
                z: 0.0,
            },
            mass: 100_000.0,
            ..default()
        },
        Vessel {
            engine_translation: -Vec3::Y * 25.0,
            sas_enabled: true,
            ..default()
        },
        AudioPlayer(assets.add(SineAudio::new(150.0))),
        ChildOf(big_space),
        children![
            (
                Name::new("Tree section"),
                Transform::from_xyz(0.0, 10.0, 1.0),
                Mesh2d(meshes.add(Mesh::from(Triangle2d::new(
                    Vec2::new(0.0, 10.0),
                    Vec2::new(-10.0, -10.0),
                    Vec2::new(10.0, -10.0),
                )))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(GREEN)))),
            ),
            (
                Name::new("ornament"),
                Transform::from_xyz(-2.0, -2.0, 1.0),
                Mesh2d(meshes.add(Mesh::from(Circle::new(2.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
            ),
            (
                Name::new("ornament"),
                Transform::from_xyz(3.0, -8.0, 1.0),
                Mesh2d(meshes.add(Mesh::from(Circle::new(2.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
            ),
            (
                Name::new("ornament"),
                Transform::from_xyz(1.0, 5.0, 1.0),
                Mesh2d(meshes.add(Mesh::from(Circle::new(2.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
            ),
            (
                Name::new("trunk"),
                Transform::from_xyz(0.0, -15.0, -1.0),
                Mesh2d(meshes.add(Mesh::from(Rectangle::new(2.0, 10.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(BROWN))),
            ),
            // (
            //     Name::new("crust"),
            //     Transform::from_xyz(0.0, -15.0, 1.0).with_rotation(Quat::from_rotation_z(PI / 2.0)),
            //     Mesh2d(meshes.add(Mesh::from(Capsule2d::new(3.0, 20.0)))),
            //     MeshMaterial2d(
            //         materials.add(ColorMaterial::from(Color::srgba(0.96, 0.69, 0.24, 1.0))),
            //     ),
            // )
        ],
    ));
    commands.spawn_vessel((
        Name::new("Flotilla Ship"),
        Transform::from_translation(translation + Vec3::X * 210.0),
        grid_cell,
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::new(13.0, 12.5, 0.0),
        },
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(BLUE_600)))),
        RigidBody {
            velocity: Vec3 {
                x: 0.0,
                y: 30.29e3,
                z: 0.0,
            },
            mass: 200_000.0,
            ..default()
        },
        Vessel {
            engine_translation: -Vec3::Y * 12.5,
            sas_enabled: true,
            ..default()
        },
        AudioPlayer(assets.add(SineAudio::new(120.0))),
        ChildOf(big_space),
        children![
            (
                Mesh2d(meshes.add(Mesh::from(Capsule2d::new(5.0, 20.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(BLUE_600)))),
            ),
            (
                Mesh2d(meshes.add(Mesh::from(Triangle2d::new(
                    Vec2::new(0.0, 8.0),
                    Vec2::new(13.0, -13.0),
                    Vec2::new(-13.0, -13.0),
                )))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(BLUE_600)))),
            ),
            (
                Mesh2d(meshes.add(Mesh::from(Segment2d::new(
                    Vec2::new(0.0, -12.0),
                    Vec2::new(0.0, -8.0)
                )))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
            ),
            (
                Transform::from_xyz(0.0, 8.0, 0.0),
                Mesh2d(meshes.add(Mesh::from(CircularSegment::new(5.0, PI / 4.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
            ),
        ],
    ));
}

fn vessel_control(
    mut query: Query<&mut Vessel>,
    action_state: Res<ActionState<VesselAction>>,
    timewarp: Res<TimeWarp>,
) {
    for mut vessel in query.iter_mut() {
        if !vessel.controlled {
            continue;
        }

        if action_state.just_pressed(&VesselAction::TogglePrecisionControls) {
            vessel.control_mode = match vessel.control_mode {
                ControlMode::Normal => ControlMode::Fine,
                ControlMode::Fine => ControlMode::Normal,
            }
        }

        let mut throttle = vessel.throttle;
        if action_state.pressed(&VesselAction::ThrottleOpen) {
            throttle = 1.0;
        }
        if action_state.pressed(&VesselAction::ThrottleClose) {
            throttle = 0.0;
        }
        if action_state.clamped_value(&VesselAction::Throttle) != 0.0 {
            let change = match vessel.control_mode {
                ControlMode::Normal => 0.01,
                ControlMode::Fine => 0.0005,
            };
            throttle = (vessel.throttle
                + change * action_state.clamped_value(&VesselAction::Throttle))
            .clamp(0.0, 1.0);
        }
        if timewarp.value > 50.0 && vessel.throttle != throttle {
            info!("Throttle is locked while Time Warp is over 50x")
        } else {
            vessel.throttle = throttle;
        }

        // TODO: Do this as angular torque instead of setting rotation directly.
        let angle: f32 = match vessel.control_mode {
            ControlMode::Normal => 200.0,
            ControlMode::Fine => 10.0,
        };
        vessel.rotate = (-angle * action_state.clamped_value(&VesselAction::Rotate)).to_radians();
        if vessel.rotate != 0.0 {
            vessel.direction_lock = None;
        }

        if action_state.pressed(&VesselAction::SasModePrograde) {
            vessel.direction_lock = Some(Direction::Prograde);
        }
        if action_state.pressed(&VesselAction::SasModeRetrograde) {
            vessel.direction_lock = Some(Direction::Retrograde);
        }
        if action_state.pressed(&VesselAction::SasModeRadial) {
            vessel.direction_lock = Some(Direction::Radial);
        }
        if action_state.pressed(&VesselAction::SasModeAntiRadial) {
            vessel.direction_lock = Some(Direction::AntiRadial);
        }
        if action_state.just_pressed(&VesselAction::SasToggle) {
            vessel.sas_enabled = !vessel.sas_enabled;
        }
    }
}

fn vessel_engine_audio(query: Query<(&Vessel, &SpatialAudioSink)>) {
    for (vessel, audiosink) in &query {
        if vessel.throttle == 0.0 {
            audiosink.pause();
        } else {
            audiosink.set_speed(vessel.throttle);
            audiosink.play();
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ParticleQueryData {
    entity: Entity,
    cell: Write<CellCoord>,
    transform: Write<Transform>,
    rigidbody: Write<RigidBody>,
    ephemeral: Write<Ephemeral>,
    color_material_handle: Read<MeshMaterial2d<ColorMaterial>>,
}

/// Applies effects of active vessel controls.
fn vessel_systems(
    mut commands: Commands,
    mut vessels: Query<(
        &mut Transform,
        &mut RigidBody,
        &Vessel,
        &CellCoord,
        &GlobalTransform,
        &SatelliteOf,
    )>,
    time: Res<Time>,
    primary_query: Query<(&GlobalTransform, &RigidBody), Without<Vessel>>,
    (mut meshes, mut materials, mut engine_particle_spawn_timer): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<ColorMaterial>>,
        ResMut<EngineParticleSpawnTimer>,
    ),
    big_space: Single<Entity, With<BigSpace>>,
    mut disabled_engine_particle_query: Query<
        ParticleQueryData,
        (With<EngineParticle>, With<Disabled>),
    >,
) {
    engine_particle_spawn_timer.0.tick(time.delta());
    let mut disabled_engine_particles = disabled_engine_particle_query.iter_mut();
    for (transform, mut rigidbody, vessel, grid_cell, global_transform, satellite_of) in
        vessels.iter_mut()
    {
        if vessel.sas_enabled {
            // TODO: Do this by applying torque and let physics solve for velocity.
            rigidbody.angular_velocity = rigidbody.angular_velocity.lerp(0.0, 0.025);
        }
        if vessel.sas_enabled
            && let Some(direction_lock) = &vessel.direction_lock
            && let Ok((primary_global_transform, primary_rigidbody)) =
                primary_query.get(satellite_of.primary())
        {
            let relative_position =
                global_transform.translation() - primary_global_transform.translation();
            let relative_velocity = rigidbody.velocity - primary_rigidbody.velocity;
            let modifier = match direction_lock {
                Direction::Prograde => 0.0,
                Direction::Retrograde => PI,
                Direction::Radial => {
                    PI / 2.0
                        * relative_velocity
                            .xy()
                            .angle_to(relative_position.xy())
                            .signum()
                }
                Direction::AntiRadial => {
                    -PI / 2.0
                        * relative_velocity
                            .xy()
                            .angle_to(relative_position.xy())
                            .signum()
                }
            };
            let rel = (transform.rotation * Vec3::Y)
                .truncate()
                .angle_to(Rot2::radians(modifier) * relative_velocity.xy());
            rigidbody.angular_velocity = if rigidbody.angular_velocity >= 2.0 {
                rigidbody.angular_velocity.lerp(0.0, 0.01)
            } else {
                rigidbody.angular_velocity
                // + 20.0
                //         * time.delta_secs()
                //         // + angle
                //         * (rel/PI)
            };
            let desired_turn_rate = 20.0;
            rigidbody.torque = rigidbody.mass * desired_turn_rate * (rel / PI);
            // transform.rotation = Quat::from_axis_angle(Vec3::Z, angle + modifier);
        } else if vessel.rotate != 0.0 {
            rigidbody.torque = rigidbody.mass * 60.0 * vessel.rotate * time.delta_secs();
            // rigidbody.angular_velocity += vessel.rotate * time.delta_secs();
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
            if engine_particle_spawn_timer.0.just_finished() {
                let translation = transform.translation
                            // One z-layer below vessel.
                            -Vec3::Z
                            // Emit from rear of vessel.
                            + transform.rotation * vessel.engine_translation;
                let velocity = rigidbody.velocity
                    + ((transform.rotation
                        * Vec3 {
                            x: rng().random_range(-0.2..0.2),
                            y: -1.0,
                            z: 0.0,
                        })
                        * (force_magnitude
                            / (rigidbody.mass
                                        * 0.004 // engine mass ejection rate as fraction of total vessel mass 
                                        * engine_particle_spawn_timer.0.duration().as_secs_f32()))
                        * time.delta_secs());
                if let Some(mut particle) = disabled_engine_particles.next() {
                    commands
                        .entity(particle.entity)
                        .remove::<Disabled>()
                        .insert(ChildOf(*big_space))
                        .insert(SatelliteOf(satellite_of.primary()));
                    *particle.cell = *grid_cell;
                    particle.transform.translation = translation;
                    particle.ephemeral.ttl.reset();
                    particle.rigidbody.velocity = velocity;
                    materials
                        .get_mut(particle.color_material_handle)
                        .unwrap()
                        .color = Color::srgba(10.0, 6.0, 1.0, 1.0);
                } else {
                    commands.spawn((
                        Name::new("engine particle"),
                        Mesh2d(meshes.add(Mesh::from(Cuboid::from_length(3.7)))),
                        Transform::from_translation(
                            transform.translation
                            // One z-layer below vessel.
                            -Vec3::Z
                            // Emit from rear of vessel.
                            + transform.rotation * vessel.engine_translation,
                        ),
                        MeshMaterial2d(materials.add(ColorMaterial {
                            color: Color::srgba(10.0, 6.0, 1.0, 1.0),
                            alpha_mode: AlphaMode2d::Blend,
                            texture: None,
                            ..default()
                        })),
                        *grid_cell,
                        // TODO: Fix this velocity
                        RigidBody {
                            velocity,
                            mass: 50.0,
                            ..default()
                        },
                        PhysicsMaterial { restituion: 0.1 },
                        NoGravity,
                        Autoscale::default(),
                        Ephemeral::new(
                            Timer::new(Duration::from_secs(5), TimerMode::Once),
                            ExpirationAction::Disable,
                            Clock::Virtual,
                        ),
                        EngineParticle,
                        Drag,
                        Collider,
                        ChildOf(*big_space),
                        SatelliteOf(satellite_of.primary()),
                    ));
                }
            }
        }
    }
}

fn animate_engine_particles(
    mut query: Query<
        (&mut Transform, &MeshMaterial2d<ColorMaterial>, &Ephemeral),
        With<EngineParticle>,
    >,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (mut transform, color_material_handle, ephemeral) in query.iter_mut() {
        transform.scale *= 1.005;
        let material = color_materials.get_mut(color_material_handle).unwrap();
        if material.color.luminance() > 1.0 {
            material.color.mix_assign(material.color.darker(0.5), 0.1);
        } else {
            let alpha = (ephemeral.ttl.fraction_remaining() * 10.0).round() / 10.0;
            material
                .color
                .mix_assign(Color::srgba(1.0, 1.0, 1.0, alpha), 1.0);
            // Psychedelic!
            // material.color = material.color.rotate_hue(10.0);
        }
    }
}

fn fire_photon(
    query: Query<(&Vessel, &CellCoord, &Transform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    big_space: Single<Entity, With<BigSpace>>,
) {
    for (vessel, grid_cell, transform) in query {
        if vessel.controlled {
            commands.spawn((
                Name::new("photon"),
                *transform,
                *grid_cell,
                Mesh2d(meshes.add(Mesh::from(Circle::new(10.0)))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(3.0, 3.0, 3.0)))),
                RigidBody {
                    mass: 1.0, // TODO: Why doesn't 0.0 work here?
                    velocity: transform.rotation * Vec3::new(0.0, SPEED_OF_LIGHT, 0.0),
                    ..default()
                },
                NoGravity,
                Autoscale::default(),
                ChildOf(*big_space),
            ));
        }
    }
}
