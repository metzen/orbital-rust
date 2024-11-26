use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use big_space::{BigSpace, GridCell};

use crate::{camera::Autoscale, lifetime::Ephemeral};

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrailTimer(Timer::from_seconds(0.2, TimerMode::Repeating)));
        app.add_systems(FixedUpdate, trail_system);
    }
}

#[derive(Component)]
pub struct Trailable;

#[derive(Resource)]
struct TrailTimer(Timer);

fn trail_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TrailTimer>,
    query: Query<
        (
            &Transform,
            &GridCell<i32>,
            &Handle<ColorMaterial>,
            // &Mesh2dHandle,
        ),
        With<Trailable>,
    >,
    big_space_query: Query<Entity, With<BigSpace>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let big_space = big_space_query.single();
        for (transform, grid_cell, material) in query.iter() {
            let color = materials.get(material).unwrap().color;
            let mut trail = commands.spawn((
                // Name::new("trail"),
                MaterialMesh2dBundle {
                    transform: Transform::from_translation(transform.translation),
                    mesh: meshes.add(Mesh::from(Circle::new(1.0))).into(),
                    material: materials.add(ColorMaterial::from(color)),
                    ..default()
                },
                Ephemeral { ttl: 60 * 30 },
                *grid_cell,
                Autoscale,
            ));
            trail.set_parent(big_space);
        }
    }
}
// options = self._simulation.get_component(RenderingOptions)
// if not (options and options.trails):
//     return
// self._elapsed_time += delta_time
// self._total_frames += 1
// # if self._elapsed_time - self._last_emit_time > 0.1:
// if self._total_frames - self._last_emit_frame > 5:
// logging.info("Creating trail entities")
// self._last_emit_time = self._elapsed_time
// self._last_emit_frame = self._total_frames
// entities = (entity for entity in self._entities if Trailable in entity)
