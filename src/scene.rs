use crate::{
    camera::{Autoscale, Focusable},
    physics::{CelestialBody, RigidBody},
    trails::Trailable,
};
use bevy::{
    color::palettes::css::{GRAY, RED, YELLOW},
    prelude::*,
    render::mesh::CircleMeshBuilder,
};
use big_space::{BigSpaceCommands, ReferenceFrame};

pub struct Planet {
    pub mass: f32,
    pub radius: f32,
}

impl Planet {
    pub const SUN: Self = Self {
        mass: 1.9891e30,
        radius: 695.7e6,
    };
    pub const MERCURY: Self = Self {
        mass: 3.303e23,
        radius: 2.4397e6,
    };
    pub const VENUS: Self = Self {
        mass: 4.869e24,
        radius: 6.0518e6,
    };
    pub const EARTH: Self = Self {
        mass: 5.976e24,
        radius: 6.37814e6,
    };
    pub const MOON: Self = Self {
        mass: 7.3e22,
        radius: 1.74e6,
    };
    pub const MARS: Self = Self {
        mass: 6.421e23,
        radius: 3.3972e6,
    };
}

// impl Default for PlanetBundle {
//     fn default() -> Self {
//         Self {
//             name: Name::new(""),
//             material_mesh_2d: MaterialMesh2dBundle::default(),
//             rigidbody: RigidBody::default(),
//             autoscale: Autoscale,
//             focusable: Focusable::default(),
//         }
//     }
// }

/// Spawn the simulation entities.
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_big_space(ReferenceFrame::<i32>::default(), |root| {
        root.spawn_spatial((
            Name::new("Sun"),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::SUN.radius)))),
            Transform::from_xyz(0.0, 0.0, 1.0),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(3.0, 3.0, 3.0)))),
            RigidBody {
                mass: Planet::SUN.mass,
                velocity: Vec3::ZERO,
                ..default()
            },
            Autoscale,
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Mercury"),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::MERCURY.radius)))),
            Transform::from_xyz(0.0, 46e9, 0.0),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(GRAY)))),
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
            Autoscale,
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Venus"),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::VENUS.radius)))),
            Transform::from_xyz(0.0, -108.2e9, 0.0),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(YELLOW)))),
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
            Autoscale,
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Earth"),
            Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                Planet::EARTH.radius,
                2000,
            )))),
            Transform::from_xyz(147.10e9, 0.0, 0.0),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb_u8(17, 145, 250)))),
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
                atmosphere_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            },
            Trailable,
            Autoscale,
            Focusable,
        ))
        // TODO: Extract this to a helper function?
        .with_children(|earth| {
            // Spawn Earth atmosphere layers.
            earth.spawn((
                Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                    Planet::EARTH.radius + 100_000.0,
                    2000,
                )))),
                // transform: Transform::from_xyz(147.10e9, 0.0, 0.0),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb_u8(
                    17 / 7,
                    145 / 7,
                    250 / 7,
                )))),
                Transform::from_xyz(0.0, 0.0, -1.0),
            ));
            earth.spawn((
                Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                    Planet::EARTH.radius + 50_000.0,
                    2000,
                )))),
                // transform: Transform::from_xyz(147.10e9, 0.0, 0.0),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb_u8(
                    17 / 5,
                    145 / 5,
                    250 / 5,
                )))),
                Transform::from_xyz(0.0, 0.0, -1.0),
            ));
            earth.spawn((
                Mesh2d(meshes.add(Mesh::from(CircleMeshBuilder::new(
                    Planet::EARTH.radius + 12_000.0,
                    2000,
                )))),
                // transform: Transform::from_xyz(147.10e9, 0.0, 0.0),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb_u8(
                    17 / 3,
                    145 / 3,
                    250 / 3,
                )))),
                Transform::from_xyz(0.0, 0.0, -1.0),
            ));
        });
        root.spawn_spatial((
            Name::new("Moon"),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::MOON.radius)))),
            Transform::from_xyz(147.10e9 + 385e6, 0.0, 0.0),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
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
            Autoscale,
            Focusable,
        ));
        root.spawn_spatial((
            Name::new("Mars"),
            Mesh2d(meshes.add(Mesh::from(Circle::new(Planet::MARS.radius)))),
            Transform::from_xyz(206.7e9, 0.0, 0.0),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(RED)))),
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
            Autoscale,
            Focusable,
        ));
    });
}
