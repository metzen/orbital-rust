use std::f64::INFINITY;
use std::f64::consts::PI;
use std::time::Duration;

use bevy::camera::primitives::Aabb;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::{Read, Write};
use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;

/// Gravitational constant.
const G: f64 = 6.67430e-11; // (N * m**2) / kg**2

pub const SPEED_OF_LIGHT: f32 = 299_792_458.0; // m/s

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(PostStartup, setup_previous_transforms);
        app.add_systems(
            FixedUpdate,
            (
                // In Verlet, use old velocity and acceleration to calc new position,
                // then use new position to calculate new acceleration and velocity.
                (kinematics, collision, gravity, drag, dynamics).chain(),
            ),
        );
    }
}

#[derive(Component)]
pub struct Collider;

#[derive(Component, Default, Reflect)]
pub struct RigidBody {
    /// The mass of an entity (in kilograms).
    pub mass: f32,
    pub velocity: Vec3,
    pub force: Vec3,
    pub acceleration: Vec3,
    pub primary: Option<Entity>,
    pub primary_force_magnitude: f32,
}

#[derive(Component, Default, Reflect)]
pub struct PhysicsMaterial {
    pub restituion: f32,
    // TODO: restitution combine modes (avg, mul, min, max)
    // TODO: pub friction: f32,
}

#[derive(Component, Default)]
pub struct CelestialBody {
    pub atmosphere_height: f32,
    pub atmosphere_density_at_sea_level: f32,
    pub atmosphere_color: Color,
    pub radius: f32,
}

impl CelestialBody {
    /// Calculate the atmospheric density for this CelestialBody at a given altitude.
    fn atmosphere_density_at_altitude(&self, altitude: f32) -> f32 {
        self.atmosphere_density_at_sea_level
            * (altitude / self.atmosphere_height)
                .min(1.0)
                .log(100.0)
                .min(0.0)
                .abs()
    }
}

#[derive(Component)]
pub struct NoGravity;

#[derive(Component)]
pub struct Drag;

fn gravitation_force(m1: f64, m2: f64, distance: DVec3) -> Vec3 {
    let unit = distance.normalize() * DVec3::new(1.0, 1.0, 0.0);
    (unit * (G * m1 * m2 / distance.length_squared())).as_vec3()
}

fn tidal_force(m1: f64, m2: f64, distance: DVec3) -> Vec3 {
    let unit = distance.normalize() * DVec3::new(1.0, 1.0, 0.0);
    (unit * (2.0 * G * m1 * m2 / distance.length().powi(3))).as_vec3()
}

#[derive(QueryData)]
#[query_data(mutable)]
struct GravityQuery {
    entity: Entity,
    grid_cell: &'static CellCoord,
    transform: &'static Transform,
    rigidbody: &'static mut RigidBody,
}

fn gravity(
    mut query: Query<GravityQuery, Without<NoGravity>>,
    grid: Single<&Grid, With<BigSpace>>,
) {
    let mut iter = query.iter_combinations_mut();
    while let Some([mut a, mut b]) = iter.fetch_next() {
        let distance = grid.grid_position_double(a.grid_cell, a.transform)
            - grid.grid_position_double(b.grid_cell, b.transform);
        let force = gravitation_force(a.rigidbody.mass.into(), b.rigidbody.mass.into(), distance);
        let tidal_force = tidal_force(a.rigidbody.mass.into(), b.rigidbody.mass.into(), distance);
        a.rigidbody.force -= force;
        let tidal_force_magnitude = tidal_force.length();
        if a.rigidbody.mass < b.rigidbody.mass
            && tidal_force_magnitude > a.rigidbody.primary_force_magnitude
        {
            a.rigidbody.primary = Some(b.entity);
            a.rigidbody.primary_force_magnitude = tidal_force_magnitude;
        }
        b.rigidbody.force += force;
        if b.rigidbody.mass < a.rigidbody.mass
            && tidal_force_magnitude > b.rigidbody.primary_force_magnitude
        {
            b.rigidbody.primary = Some(a.entity);
            b.rigidbody.primary_force_magnitude = tidal_force_magnitude;
        }
    }
    for mut item in query.iter_mut() {
        item.rigidbody.primary_force_magnitude = 0.0;
    }
}

#[derive(QueryData)]
struct DragPrimaryQueryData {
    rigidbody: Read<RigidBody>,
    celestial_body: Read<CelestialBody>,
    aabb: Read<Aabb>,
    transform: Read<Transform>,
    cell: Read<CellCoord>,
}

fn drag(
    mut query: Query<(&mut RigidBody, &Transform, &CellCoord, &Aabb), With<Drag>>,
    primary_query: Query<DragPrimaryQueryData, Without<Drag>>,
    grid: Single<&Grid, With<BigSpace>>,
    time: Res<Time>,
) {
    for (mut rigidbody, transform, cell, aabb) in query.iter_mut() {
        // D = Cd * A * .5 * r * V^2
        // TODO: This is just currently hard coded for the vessel engine particles.
        let drag_coefficient = 0.5;
        let area: f32;
        let velocity: Vec3;
        let density: f32;
        if let Some(primary_id) = rigidbody.primary
            && let Ok(primary) = primary_query.get(primary_id)
        {
            velocity = rigidbody.velocity - primary.rigidbody.velocity;
            area = aabb.half_extents.y.lerp(
                aabb.half_extents.x,
                velocity
                    .normalize_or_zero()
                    .dot(transform.rotation * Vec3::Y)
                    .abs(),
            ) * 2.0;
            let distance = (grid.grid_position_double(cell, transform)
                - grid.grid_position_double(primary.cell, primary.transform))
            .length();
            let altitude = distance as f32 - primary.aabb.half_extents.x;
            density = primary
                .celestial_body
                .atmosphere_density_at_altitude(altitude);
        } else {
            velocity = rigidbody.velocity;
            density = 0.0;
            area = 1.0;
        };

        let b = drag_coefficient * area * 0.5 * density;

        let v_mag_at_t = 1.0 / (1.0 / velocity.length() + b * time.delta_secs() / rigidbody.mass);
        let delta_v = v_mag_at_t - velocity.length();

        let drag_force_magnitude = rigidbody.mass * delta_v / time.delta_secs();
        let drag_force = drag_force_magnitude * velocity.normalize_or_zero() * 1.0;
        rigidbody.force += drag_force;
    }
}

pub fn dynamics(mut query: Query<&mut RigidBody>, time: Res<Time>) {
    for mut rigidbody in query.iter_mut() {
        let delta_time = time.delta_secs();
        let old_acceleration = rigidbody.acceleration;
        let new_acceleration = rigidbody.force / rigidbody.mass;
        // TODO: inverse mass
        rigidbody.acceleration = new_acceleration;
        rigidbody.velocity += 0.5 * (old_acceleration + new_acceleration) * delta_time;
        rigidbody.velocity = rigidbody.velocity.clamp_length(0.0, SPEED_OF_LIGHT);
        rigidbody.force = Vec3::ZERO;
    }
}

fn kinematics(mut query: Query<(&mut Transform, &mut RigidBody)>, time: Res<Time>) {
    for (mut transform, rigidbody) in query.iter_mut() {
        let dt = time.delta_secs();
        // let velocity = rigidbody.velocity;
        // let acceleration = rigidbody.acceleration;
        // TODO: High precision f64 velocity and accel?
        // rigidbody.transform.translation += dt * (velocity + 0.5 * acceleration * dt);
        transform.translation += dt * (rigidbody.velocity + 0.5 * rigidbody.acceleration * dt);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CollisionQueryData {
    entity: Entity,
    name: Read<Name>,
    transform: Write<Transform>,
    grid_cell: Write<CellCoord>,
    rigidbody: Write<RigidBody>,
    aabb: Read<Aabb>,
}

fn collision(
    mut query: Query<CollisionQueryData, With<Collider>>,
    grid: Single<&Grid, With<BigSpace>>,
) {
    let mut iter = query.iter_combinations_mut();
    while let Some([a, b]) = iter.fetch_next() {
        let (mut primary, mut secondary) = if a.rigidbody.mass > b.rigidbody.mass {
            (a, b)
        } else {
            (b, a)
        };

        let primary_position = grid.grid_position_double(&primary.grid_cell, &primary.transform);
        let secondary_position =
            grid.grid_position_double(&secondary.grid_cell, &secondary.transform);
        let relative_position = (secondary_position - primary_position) * DVec3::new(1.0, 1.0, 0.0);
        let relative_position_normalized = relative_position.normalize();

        // TODO: Handle non square/circle shapes.
        let collision_distance =
            (primary.aabb.half_extents.y + secondary.aabb.half_extents.y) as f64;
        let diff = relative_position.length_squared() - collision_distance * collision_distance;
        if diff < 0.0 {
            debug!(
                "Collision between {} and {} ({} < {})",
                primary.name,
                secondary.name,
                relative_position.length(),
                collision_distance
            );
            let relative_velocity = primary.rigidbody.velocity - secondary.rigidbody.velocity;
            // TODO: Replace 0.2 with restituion from colliding entities.
            let collision_speed =
                relative_velocity.dot(relative_position_normalized.as_vec3()) * 0.2;
            if collision_speed < 0.0 {
                // Already moving away from each other, so just ignore for now.
                continue;
            }
            let overlap = collision_distance - relative_position.length();

            let (new_grid_cell, new_translation) = grid
                .translation_to_grid(secondary_position + relative_position_normalized * overlap);

            *secondary.grid_cell = new_grid_cell;
            secondary.transform.translation.x = new_translation.x;
            secondary.transform.translation.y = new_translation.y;

            let impulse =
                2.0 * collision_speed / (primary.rigidbody.mass + secondary.rigidbody.mass);
            primary.rigidbody.velocity -=
                impulse * secondary.rigidbody.mass * relative_position_normalized.as_vec3();
            secondary.rigidbody.velocity +=
                impulse * primary.rigidbody.mass * relative_position_normalized.as_vec3();
        }
    }
}

/// A 64-bit floating-point type representing an angle in radians.
#[derive(Deref)]
struct Rad64(f64);

impl Rad64 {
    fn to_f64(&self) -> f64 {
        self.0
    }
}

#[derive(Debug)]
pub enum OrbitShape {
    Circle,
    Ellipse,
    Parabola,
    Hyperbola,
}

pub struct Orbit {
    pub position: DVec2,
    pub velocity: DVec2,
    pub μ: f64,
    pub semi_major_axis: f64,
    pub semi_minor_axis: f64,
    pub eccentricity: f64,
    pub period: Duration,
    pub apoapsis: f64,
    pub periapsis: f64,
    // https://space.stackexchange.com/questions/2562/2d-orbital-path-from-state-vectors
    // https://en.wikipedia.org/wiki/Eccentricity_vector
    pub eccentricity_vector: DVec2,
}

impl Orbit {
    pub fn new(position: DVec2, velocity: DVec2, primary_mass: f32, secondary_mass: f32) -> Self {
        let μ = G * (primary_mass as f64 + secondary_mass as f64);
        let position_length = position.length();
        let velocity_length_squared = velocity.length_squared();
        let orbital_energy = velocity_length_squared / 2.0 - μ / position_length;
        let angular_momentum = position.perp_dot(velocity);
        let eccentricity =
            (1.0 + ((2.0 * orbital_energy * angular_momentum.powi(2)) / (μ.powi(2)))).sqrt();
        let eccentricity_squared = eccentricity.powi(2);
        let semi_major_axis = if eccentricity < 1.0 {
            -(μ * position_length / (position_length * velocity_length_squared - (2.0 * μ)))
        } else {
            (angular_momentum.powi(2) / μ) * (1.0 / (eccentricity_squared - 1.0))
        };
        let semi_minor_axis = if eccentricity < 1.0 {
            semi_major_axis * (1.0 - eccentricity_squared).sqrt()
        } else {
            semi_major_axis * (eccentricity_squared - 1.0).sqrt()
        };
        let period = Duration::try_from_secs_f64(2.0 * PI * (semi_major_axis.powi(3) / μ).sqrt())
            .unwrap_or(Duration::MAX);
        let apoapsis = match eccentricity {
            1.0..INFINITY => -semi_major_axis * (eccentricity + 1.0),
            _ => semi_major_axis * (1.0 + eccentricity),
        };
        let periapsis = match eccentricity {
            1.0..INFINITY => semi_major_axis * (eccentricity - 1.0),
            _ => semi_major_axis * (1.0 - eccentricity),
        };
        let eccentricity_vector = (velocity_length_squared / μ - 1.0 / position_length) * position
            - ((position.dot(velocity)) / μ) * velocity;
        Self {
            position,
            velocity,
            μ,
            semi_major_axis,
            semi_minor_axis,
            eccentricity,
            period,
            apoapsis,
            periapsis,
            eccentricity_vector,
        }
    }

    pub fn shape(&self) -> OrbitShape {
        match self.eccentricity {
            0.0 => OrbitShape::Circle,
            0.0..1.0 => OrbitShape::Ellipse,
            1.0 => OrbitShape::Parabola,
            1.0..INFINITY => OrbitShape::Hyperbola,
            _ => panic!("Unexpected eccentricity: {}", self.eccentricity),
        }
    }

    // Some implementations taken from
    // https://stackoverflow.com/questions/71863525/calculating-2d-orbital-paths-in-newtonian-gravity-simulation
    // but these might be a little off.

    fn true_anomaly(&self) -> Rad64 {
        // https://en.wikipedia.org/wiki/True_anomaly
        let value = (((self.eccentricity_vector.dot(self.position))
            / (self.eccentricity * self.position.length()))
        .clamp(-1.0, 1.0))
        .acos();
        Rad64(if self.position.dot(self.velocity) < 0.0 {
            2.0 * PI - value
        } else {
            value
        })
    }

    fn mean_anomaly(&self) -> Rad64 {
        // https://en.wikipedia.org/wiki/Mean_anomaly
        let eccentric_anomaly = self.eccentric_anomaly();
        Rad64(eccentric_anomaly.to_f64() - self.eccentricity * eccentric_anomaly.sin())
    }

    fn eccentric_anomaly(&self) -> Rad64 {
        // https://en.wikipedia.org/wiki/Eccentric_anomaly
        let true_anomaly = self.true_anomaly();
        let value = f64::atan2(
            (1.0 - self.eccentricity.powi(2)).sqrt() * (true_anomaly).sin(),
            self.eccentricity + (true_anomaly).cos(),
        );
        // Keep in range [0, 2pi] instead of [-pi, pi] radians.
        Rad64(if value < 0.0 { 2.0 * PI + value } else { value })
    }

    fn orbital_time_at(&self, eccentric_anomaly: Rad64) -> Duration {
        Duration::try_from_secs_f64(
            (self.semi_major_axis.powi(3) / self.μ).sqrt()
                * (eccentric_anomaly.to_f64() - self.eccentricity * eccentric_anomaly.sin()),
        )
        .unwrap_or(Duration::MAX)
    }

    fn time_since_periapsis(&self) -> Duration {
        Duration::try_from_secs_f64(
            self.mean_anomaly().to_f64() * self.period.as_secs_f64() / (2.0 * PI),
        )
        .unwrap_or(Duration::MAX)
    }

    pub fn time_until_apoapsis(&self) -> Duration {
        if self.eccentricity >= 1.0 {
            Duration::ZERO
        } else {
            let time_since_periapsis = self.time_since_periapsis();
            let time_at_apoapsis = self.orbital_time_at(Rad64(PI));
            if time_since_periapsis < time_at_apoapsis {
                time_at_apoapsis - time_since_periapsis
            } else {
                self.period - time_since_periapsis + time_at_apoapsis
            }
        }
    }

    pub fn time_to_periapsis(&self) -> Duration {
        if self.eccentricity >= 1.0 {
            Duration::ZERO
        } else {
            self.period - self.time_since_periapsis()
        }
    }
}
