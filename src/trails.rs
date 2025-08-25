use std::time::Duration;

use bevy::{ecs::query::QueryData, prelude::*};
use big_space::{floating_origins::BigSpace, grid::cell::GridCell};

use crate::{camera::Autoscale, lifetime::Ephemeral};

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrailTimer(Timer::from_seconds(0.2, TimerMode::Repeating)));
        app.insert_resource(TrailsOptions { enabled: true });
        app.init_resource::<TrailAssets>();
        app.add_systems(FixedUpdate, trail_system);
    }
}

#[derive(Component)]
pub struct Trailable;

#[derive(Resource)]
struct TrailTimer(Timer);

#[derive(Resource)]
struct TrailsOptions {
    enabled: bool,
}

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
    grid_cell: &'static GridCell,
    material: &'static MeshMaterial2d<ColorMaterial>,
}

fn trail_system(
    options: Res<TrailsOptions>,
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TrailTimer>,
    assets: Res<TrailAssets>,
    query: Query<TrailableQuery, With<Trailable>>,
    big_space: Single<Entity, With<BigSpace>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !options.enabled {
        return;
    }
    if timer.0.tick(time.delta()).just_finished() {
        for trailable in query.iter() {
            let color = materials.get(trailable.material).unwrap().color;
            commands.spawn((
                Transform::from_translation(trailable.transform.translation),
                *trailable.grid_cell,
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(materials.add(ColorMaterial {
                    color,
                    alpha_mode: bevy::sprite::AlphaMode2d::Blend,
                    ..default()
                })),
                Ephemeral {
                    ttl: Timer::new(Duration::from_secs(5), TimerMode::Once),
                },
                Autoscale::default(),
                ChildOf(*big_space),
            ));
        }
    }
}
