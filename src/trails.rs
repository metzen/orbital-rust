use std::time::Duration;

use bevy::{
    ecs::{
        entity_disabling::Disabled,
        query::QueryData,
        system::lifetimeless::{Read, Write},
    },
    prelude::*,
    sprite_render::AlphaMode2d,
};
use big_space::{floating_origins::BigSpace, grid::cell::CellCoord};

use crate::{
    camera::Autoscale,
    lifetime::{Clock, Ephemeral, ExpirationAction},
};

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrailTimer(Timer::from_seconds(0.01, TimerMode::Repeating)));
        app.insert_resource(TrailsOptions { enabled: false });
        app.init_resource::<TrailAssets>();
        app.add_systems(
            Update,
            trail_system.run_if(|opts: Res<TrailsOptions>| opts.enabled),
        );
    }
}

#[derive(Component)]
pub struct Trailable;

#[derive(Component)]
pub struct TrailMarker;

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
struct TrailableQueryData {
    transform: &'static Transform,
    grid_cell: &'static CellCoord,
    material: &'static MeshMaterial2d<ColorMaterial>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct TrailMarkerQueryData {
    entity: Entity,
    grid_cell: Write<CellCoord>,
    transform: Write<Transform>,
    ephemeral: Write<Ephemeral>,
    color_material_handle: Read<MeshMaterial2d<ColorMaterial>>,
}

type TrailableQuery<'world, 'state> = Query<'world, 'state, TrailableQueryData, With<Trailable>>;
type DisabledTrailMarkerQuery<'world, 'state> =
    Query<'world, 'state, TrailMarkerQueryData, (With<TrailMarker>, With<Disabled>)>;

fn trail_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TrailTimer>,
    assets: Res<TrailAssets>,
    big_space: Single<Entity, With<BigSpace>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    (query, mut disabled_trail_marker_query): (TrailableQuery, DisabledTrailMarkerQuery),
) {
    // TODO: Prevent the system from scheduling when disabled instead.
    let mut disabled_trail_markers = disabled_trail_marker_query.iter_mut();
    if timer.0.tick(time.delta()).just_finished() {
        for trailable in query.iter() {
            let color = materials.get(trailable.material).unwrap().color;
            if let Some(mut disabled_trail_marker) = disabled_trail_markers.next() {
                commands
                    .entity(disabled_trail_marker.entity)
                    .remove::<Disabled>()
                    .insert(ChildOf(*big_space));
                *disabled_trail_marker.grid_cell = *trailable.grid_cell;
                disabled_trail_marker.transform.translation = trailable.transform.translation;
                materials
                    .get_mut(disabled_trail_marker.color_material_handle)
                    .unwrap()
                    .color = color;
                disabled_trail_marker.ephemeral.ttl.reset();
            } else {
                commands.spawn((
                    Name::new("trail marker"),
                    TrailMarker,
                    Transform::from_translation(trailable.transform.translation),
                    *trailable.grid_cell,
                    Mesh2d(assets.mesh.clone()),
                    MeshMaterial2d(materials.add(ColorMaterial {
                        color,
                        alpha_mode: AlphaMode2d::Blend,
                        ..default()
                    })),
                    Ephemeral::new(
                        Timer::new(Duration::from_secs(2), TimerMode::Once),
                        ExpirationAction::Disable,
                        Clock::Real,
                    ),
                    Autoscale::default(),
                    ChildOf(*big_space),
                ));
            }
        }
    }
}
