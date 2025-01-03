use bevy::{ecs::query::QueryData, prelude::*};

use crate::{lifetime::Ephemeral, scene::Planet, timewarp::*};

/// Gravitational constant.
const G: f64 = 6.67430e-11; // (N * m**2) / kg**2

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(PostStartup, setup_previous_transforms);
        app.add_systems(
            FixedUpdate,
            (
                // In Verlet, use old velocity and acceleration to calc new position,
                // then use new position to calculate new acceleration and velocity.
                (previous_transform_sync, kinematics, gravity, dynamics).chain(),
                drag,
                // collision,
            ),
        );
        app.add_systems(Update, transform_sync);
    }
}

#[derive(Component, Default)]
pub struct RigidBody {
    pub transform: Transform,
    pub mass: f32,
    pub velocity: Vec3,
    pub force: Vec3,
    pub acceleration: Vec3,
    pub primary: Option<Entity>,
    pub primary_force_magnitude: f32,
}

#[derive(Component)]
pub struct PreviousTransform(Transform);

#[derive(Component, Default)]
pub struct CelestialBody {
    pub atmosphere_height: f32,
    pub atmosphere_color: Color,
}

#[derive(Component)]
pub struct NoGravity;

fn gravitation_force(m1: f64, m2: f64, distance: Vec3) -> Vec3 {
    let unit = distance.normalize();
    unit * (G * m1 * m2 / distance.length_squared() as f64) as f32
}

#[derive(QueryData)]
#[query_data(mutable)]
struct GravityQuery {
    entity: Entity,
    global_transform: &'static GlobalTransform,
    rigidbody: &'static mut RigidBody,
}

fn gravity(mut query: Query<GravityQuery, Without<NoGravity>>) {
    let mut iter = query.iter_combinations_mut();
    while let Some([mut a, mut b]) = iter.fetch_next() {
        let force = gravitation_force(
            a.rigidbody.mass.into(),
            b.rigidbody.mass.into(),
            a.global_transform.translation() - b.global_transform.translation(),
        );
        let force_magnitude = force.length();
        a.rigidbody.force -= force;
        if force_magnitude > a.rigidbody.primary_force_magnitude {
            a.rigidbody.primary = Some(b.entity);
        }
        b.rigidbody.force += force;
        if force_magnitude > b.rigidbody.primary_force_magnitude {
            b.rigidbody.primary = Some(a.entity);
        }
    }
}

fn drag(mut query: Query<&mut RigidBody, With<Ephemeral>>) {
    for mut rigidbody in query.iter_mut() {
        // let primary_transform = world.get_mut::<Transform>(primary).unwrap();
        // let Some(primary) = rigidbody.primary else { todo!(); };
        // let primary_rigidbody = world.get::<RigidBody>(primary).unwrap();
        // rigidbody.velocity = rigidbody.velocity.lerp(primary_rigidbody.velocity, 0.05);
        // TODO: This is just currently hard coded for the vessel engine particles.
        let vel = Vec3 {
            x: 0.0,
            y: 30.29e3,
            z: 0.0,
        };
        rigidbody.velocity = rigidbody.velocity.lerp(vel, 0.2);
    }
}

pub fn dynamics(mut query: Query<&mut RigidBody>, time: Res<Time>, time_warp: Res<TimeWarp>) {
    for mut rigidbody in query.iter_mut() {
        let delta_time = time.delta_secs() * time_warp.value;
        let old_acceleration = rigidbody.acceleration;
        let new_acceleration = rigidbody.force / rigidbody.mass;
        // TODO: inverse mass
        rigidbody.acceleration = new_acceleration;
        rigidbody.velocity += 0.5 * (old_acceleration + new_acceleration) * delta_time;
        rigidbody.force = Vec3::ZERO;
    }
}

fn kinematics(
    mut query: Query<(&mut Transform, &mut RigidBody)>,
    time: Res<Time>,
    time_warp: Res<TimeWarp>,
) {
    for (mut transform, mut rigidbody) in query.iter_mut() {
        let dt = time.delta_secs() * time_warp.value;
        let velocity = rigidbody.velocity;
        let acceleration = rigidbody.acceleration;
        // rigidbody.transform.translation += dt * (velocity + 0.5 * acceleration * dt);
        transform.translation += dt * (rigidbody.velocity + 0.5 * rigidbody.acceleration * dt);
    }
}

fn previous_transform_sync(mut query: Query<(&mut PreviousTransform, &RigidBody)>) {
    for (mut prev, rigidbody) in &mut query {
        prev.0 = rigidbody.transform;
    }
}

fn setup_previous_transforms(mut commands: Commands, mut query: Query<(Entity, &RigidBody)>) {
    for (entity, rigidbody) in &mut query {
        commands
            .entity(entity)
            .insert(PreviousTransform(rigidbody.transform));
    }
}

/// Syncs RigidBody Transform to main Transform component.
fn transform_sync(
    mut query: Query<(&mut Transform, &PreviousTransform, &RigidBody)>,
    time: Res<Time<Fixed>>,
) {
    for (mut transform, previous_transform, rigidbody) in &mut query {
        transform.translation = previous_transform
            .0
            .translation
            .lerp(rigidbody.transform.translation, time.overstep_fraction());
    }
}

fn collision(mut query: Query<(&mut Transform, &mut RigidBody)>) {
    // # TODO: Remove the 0.9 multiplier hack and fix the rendering instead.
    // # TODO: Move shape from renderable to collision.

    let mut iter = query.iter_combinations_mut();
    while let Some([(mut at, mut ar), (mut bt, mut br)]) = iter.fetch_next() {
        // let Some(primary) = rigidbody.primary else {return};
        // let primary_transform = world.get_mut::<Transform>(primary).unwrap();
        // let primary_rigidbody = world.get_mut::<RigidBody>(primary).unwrap();
        let (pt, pr, mut st, mut sr) = if ar.mass > br.mass {
            (at, ar, bt, br)
        } else {
            (bt, br, at, ar)
        };
        let relative_position = st.translation - pt.translation;

        // collision_distance = (
        //     cast(Circle, rigidbody.primary[Renderable].shape).radius
        //     + cast(Circle, entity[Renderable].shape).radius * 0.9
        // )
        // TODO: Remove this hardcoded.
        let collision_distance = Planet::EARTH.radius + 10.0;
        let diff = relative_position.length_squared() - collision_distance * collision_distance;
        if diff < 0.0 {
            // Landed -- prevent moving below surface.
            st.translation = pt.translation + relative_position.normalize() * collision_distance;
        }
        if diff <= collision_distance * 0.05 {
            // # Simulate the effect of a normal force.
            // # rigidbody.velocity.update_from(rigidbody.primary[Rigidbody].velocity)
            sr.velocity.y = pr.velocity.y;
            sr.acceleration += Vec3 {
                x: 0.0,
                y: 9.81,
                z: 0.0,
            }
        }
    }
}
