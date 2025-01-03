use bevy::{ecs::query::QueryData, prelude::*};
use big_space::{BigSpace, GridCell};

use crate::{camera::Autoscale, lifetime::Ephemeral};

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrailTimer(Timer::from_seconds(0.2, TimerMode::Repeating)));
        app.init_resource::<TrailAssets>();
        app.add_systems(FixedUpdate, trail_system);
    }
}

#[derive(Component)]
pub struct Trailable;

#[derive(Resource)]
struct TrailTimer(Timer);

#[derive(Resource)]
struct TrailAssets {
    mesh: Handle<Mesh>,
}

impl FromWorld for TrailAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: world
                .resource_mut::<Assets<Mesh>>()
                .add(Mesh::from(Circle::new(1.0))),
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct TrailableQuery {
    transform: &'static Transform,
    grid_cell: &'static GridCell<i32>,
    material: &'static MeshMaterial2d<ColorMaterial>,
}

fn trail_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TrailTimer>,
    assets: Res<TrailAssets>,
    query: Query<TrailableQuery, With<Trailable>>,
    big_space: Single<Entity, With<BigSpace>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for trailable in query.iter() {
            let color = materials.get(trailable.material).unwrap().color;
            let mut trail = commands.spawn((
                Transform::from_translation(trailable.transform.translation),
                *trailable.grid_cell,
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(materials.add(ColorMaterial {
                    color,
                    alpha_mode: bevy::sprite::AlphaMode2d::Blend,
                    ..default()
                })),
                Ephemeral { ttl: 60 * 20 },
                Autoscale,
            ));
            trail.set_parent(*big_space);
        }
    }
}
