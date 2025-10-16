use crate::{
    camera::{Autoscale, Focusable},
    physics::{CelestialBody, PhysicsMaterial, RigidBody},
    trails::Trailable,
};
use bevy::{
    color::palettes::css::{DARK_BLUE, GRAY, LIGHT_BLUE, MAGENTA, RED},
    prelude::*,
    mesh::CircleMeshBuilder,
};
use big_space::{commands::BigSpaceCommands, prelude::BigSpace};

pub struct Planet {
    pub mass: f32,
    pub radius: f32,
    pub color: Color,
}

impl Planet {
    pub const SUN: Self = Self {
        mass: 1.9891e30,
        radius: 695.7e6,
        color: Color::srgb(3.0, 3.0, 3.0),
    };
    pub const MERCURY: Self = Self {
        mass: 3.303e23,
        radius: 2.4397e6,
        color: Color::Srgba(GRAY),
    };
    pub const VENUS: Self = Self {
        mass: 4.869e24,
        radius: 6.0518e6,
        color: Color::srgb(0.75, 0.62, 0.43),
    };
    pub const EARTH: Self = Self {
        mass: 5.976e24,
        radius: 6.37814e6,
        color: Color::srgb(0.06, 0.57, 0.98),
    };
    pub const MOON: Self = Self {
        mass: 7.3e22,
        radius: 1.74e6,
        color: Color::WHITE,
    };
    pub const MARS: Self = Self {
        mass: 6.421e23,
        radius: 3.3972e6,
        color: Color::Srgba(RED),
    };
    pub const PHOBOS: Self = Self {
        mass: 1.06e16,
        radius: 11.08e3,
        color: Color::srgb(0.78, 0.71, 0.65),
    };
    pub const DEIMOS: Self = Self {
        mass: 1.51e15,
        radius: 6.2e3,
        color: Color::srgb(0.87, 0.72, 0.58),
    };
    pub const JUPITER: Self = Self {
        mass: 1.9e27,
        radius: 71.492e6,
        color: Color::Srgba(MAGENTA),
    };
    pub const SATURN: Self = Self {
        mass: 5.688e26,
        radius: 60.268e6,
        color: Color::srgb(0.83, 0.74, 0.62),
    };
    pub const URANUS: Self = Self {
        mass: 8.686e25,
        radius: 25.559e6,
        color: Color::Srgba(LIGHT_BLUE),
    };
    pub const NEPTUNE: Self = Self {
        mass: 1.024e26,
        radius: 24.746e6,
        color: Color::Srgba(DARK_BLUE),
    };
    pub const PROXIMA_CENTAURI: Self = Self {
        mass: 2.428e29,
        radius: 107.277e6,
        color: Color::srgb(3.0, 3.0, 3.0),
    };
}

/// Spawn the simulation entities.
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_big_space_default(|root| {
        root.spawn_spatial((
            Name::new("Sun"),
            Transform::from_xyz(0.0, 0.0, 10.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::SUN.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::SUN.color))),
            RigidBody {
                mass: Planet::SUN.mass,
                velocity: Vec3::ZERO,
                ..default()
            },
            Autoscale::new(3.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Mercury"),
            Transform::from_xyz(0.0, 46e9, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::MERCURY.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::MERCURY.color))),
            RigidBody {
                mass: Planet::MERCURY.mass,
                velocity: Vec3 {
                    x: -59_000.0,
                    y: 0.0,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Venus"),
            Transform::from_xyz(0.0, -108.2e9, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::VENUS.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::VENUS.color))),
            RigidBody {
                mass: Planet::VENUS.mass,
                velocity: Vec3 {
                    x: 35_000.0,
                    y: 0.0,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Earth"),
            Transform::from_xyz(147.10e9, 0.0, 0.0),
            Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                Planet::EARTH.radius,
                4000,
            )))),
            PhysicsMaterial { restituion: 0.0 },
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::EARTH.color))),
            RigidBody {
                velocity: Vec3 {
                    x: 0.0,
                    y: 30.29e3,
                    z: 0.0,
                },
                mass: Planet::EARTH.mass,
                ..default()
            },
            CelestialBody {
                atmosphere_height: 100_000.0,
                atmosphere_density_at_sea_level: 1.225, // kg/m³
                atmosphere_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                radius: Planet::EARTH.radius,
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ))
        // TODO: Extract this to a helper function?
        .with_children(|earth| {
            // Spawn Earth atmosphere layers.
            earth.spawn((
                Transform::from_xyz(0.0, 0.0, -1.0),
                Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                    Planet::EARTH.radius + 100_000.0,
                    2000,
                )))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb_u8(
                    17 / 7,
                    145 / 7,
                    250 / 7,
                )))),
            ));
            earth.spawn((
                Transform::from_xyz(0.0, 0.0, -1.0),
                Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                    Planet::EARTH.radius + 50_000.0,
                    2000,
                )))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb_u8(
                    17 / 5,
                    145 / 5,
                    250 / 5,
                )))),
            ));
            earth.spawn((
                Transform::from_xyz(0.0, 0.0, -1.0),
                Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                    Planet::EARTH.radius + 12_000.0,
                    2000,
                )))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb_u8(
                    17 / 3,
                    145 / 3,
                    250 / 3,
                )))),
            ));
        });
        root.spawn_spatial((
            Name::new("Moon"),
            Transform::from_xyz(147.10e9 + 385e6, 0.0, -1.0),
            Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                Planet::MOON.radius,
                2000,
            )))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::MOON.color))),
            RigidBody {
                mass: Planet::MOON.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 30.29e3 + 1.022e3,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(1.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Mars"),
            Transform::from_xyz(206.7e9, 0.0, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::MARS.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::MARS.color))),
            RigidBody {
                mass: Planet::MARS.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 26.5e3,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Phobos"),
            Transform::from_xyz(206.7e9 + 9_376e3, 0.0, -1.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::PHOBOS.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::PHOBOS.color))),
            RigidBody {
                mass: Planet::PHOBOS.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 26.5e3 + 2.1704e3,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(1.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Deimos"),
            Transform::from_xyz(206.7e9 + 23_455e3, 0.0, -1.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::DEIMOS.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::DEIMOS.color))),
            RigidBody {
                mass: Planet::DEIMOS.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 26.5e3 + 1.352e3,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(1.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Jupiter"),
            Transform::from_xyz(740.595e9, 0.0, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::JUPITER.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::JUPITER.color))),
            RigidBody {
                mass: Planet::JUPITER.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 13.72e3,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Saturn"),
            Transform::from_xyz(1352.55e9, 0.0, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::SATURN.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::SATURN.color))),
            RigidBody {
                mass: Planet::MARS.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 10.14e3,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ))
        .with_child((
            Mesh2d(meshes.add(Mesh::from(Annulus::new(
                Planet::SATURN.radius * 1.4,
                Planet::SATURN.radius * 1.8,
            )))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.44, 0.44, 0.44)))),
        ));
        root.spawn_spatial((
            Name::new("Uranus"),
            Transform::from_xyz(2735.56e9, 0.0, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::URANUS.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::URANUS.color))),
            RigidBody {
                mass: Planet::MARS.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 7130.0,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Neptune"),
            Transform::from_xyz(4471.05e9, 0.0, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::NEPTUNE.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::NEPTUNE.color))),
            RigidBody {
                mass: Planet::MARS.mass,
                velocity: Vec3 {
                    x: 0.0,
                    y: 5470.0,
                    z: 0.0,
                },
                ..default()
            },
            Trailable,
            Autoscale::new(2.0),
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Proxima Centauri"),
            Transform::from_xyz(4.017499e16, 0.0, 0.0),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::PROXIMA_CENTAURI.radius)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Planet::PROXIMA_CENTAURI.color))),
            RigidBody {
                mass: Planet::PROXIMA_CENTAURI.mass,
                velocity: Vec3::ZERO,
                ..default()
            },
            Autoscale::new(3.0),
        ));
    });
}

pub fn add_name_to_big_space(
    mut commands: Commands,
    big_space_entity: Single<Entity, With<BigSpace>>,
) {
    commands
        .entity(*big_space_entity)
        .insert(Name::new("BigSpace"));
}
