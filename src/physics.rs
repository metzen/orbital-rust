use bevy::{
    ecs::{
        query::QueryData,
        system::lifetimeless::{Read, Write},
    },
    math::DVec3,
    prelude::*,
    render::primitives::Aabb,
};
use big_space::{
    floating_origins::BigSpace,
    grid::{Grid, cell::GridCell},
};

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

#[derive(Component, Default, Reflect)]
pub struct RigidBody {
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
    pub atmosphere_color: Color,
    pub radius: f32,
}

#[derive(Component)]
pub struct NoGravity;

#[derive(Component)]
pub struct Drag;

fn gravitation_force(m1: f64, m2: f64, distance: DVec3) -> Vec3 {
    let unit = distance.normalize() * DVec3::new(1.0, 1.0, 0.0);
    (unit * (G * m1 * m2 / distance.length_squared())).as_vec3()
}

#[derive(QueryData)]
#[query_data(mutable)]
struct GravityQuery {
    entity: Entity,
    grid_cell: &'static GridCell,
    transform: &'static Transform,
    rigidbody: &'static mut RigidBody,
}

fn gravity(
    mut query: Query<GravityQuery, Without<NoGravity>>,
    grid: Single<&Grid, With<BigSpace>>,
) {
    let mut iter = query.iter_combinations_mut();
    while let Some([mut a, mut b]) = iter.fetch_next() {
        let force = gravitation_force(
            a.rigidbody.mass.into(),
            b.rigidbody.mass.into(),
            grid.grid_position_double(a.grid_cell, a.transform)
                - grid.grid_position_double(b.grid_cell, b.transform),
        );
        let force_magnitude = force.length();
        a.rigidbody.force -= force;
        if force_magnitude > a.rigidbody.primary_force_magnitude {
            a.rigidbody.primary = Some(b.entity);
            a.rigidbody.primary_force_magnitude = force_magnitude;
        }
        b.rigidbody.force += force;
        if force_magnitude > b.rigidbody.primary_force_magnitude {
            b.rigidbody.primary = Some(a.entity);
            b.rigidbody.primary_force_magnitude = force_magnitude;
        }
    }
}

fn drag(
    mut query: Query<&mut RigidBody, With<Drag>>,
    rigidbody_query: Query<&RigidBody, Without<Drag>>,
    time: Res<Time>,
) {
    for mut rigidbody in query.iter_mut() {
        // let primary_transform = world.get_mut::<Transform>(primary).unwrap();
        // let Some(primary) = rigidbody.primary else { todo!(); };
        // let primary_rigidbody = world.get::<RigidBody>(primary).unwrap();
        // rigidbody.velocity = rigidbody.velocity.lerp(primary_rigidbody.velocity, 0.05);
        // TODO: This is just currently hard coded for the vessel engine particles.
        // D = Cd * A * .5 * r * V^2
        let vel = Vec3 {
            x: 0.0,
            y: 30.29e3,
            z: 0.0,
        };
        let drag_coefficient = 0.5;
        let area = 1.0;
        let density_of_air_at_sea_lvl = 1.225; // kg/m³
        let velocity = if let Some(primary) = rigidbody.primary
            && let Ok(primary_body) = rigidbody_query.get(primary)
        {
            rigidbody.velocity - primary_body.velocity
        } else {
            rigidbody.velocity
        };

        let b = drag_coefficient * area * 0.5 * density_of_air_at_sea_lvl;

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
    grid_cell: Write<GridCell>,
    rigidbody: Write<RigidBody>,
    aabb: Read<Aabb>,
}

fn collision(mut query: Query<CollisionQueryData>, grid: Single<&Grid, With<BigSpace>>) {
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
