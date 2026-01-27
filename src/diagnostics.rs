use bevy::camera::visibility::RenderLayers;
use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;

use crate::camera::HIGH_RES_LAYER;

const FONT_SIZE: f32 = 11.0;

#[derive(Component)]
struct Fps;

#[derive(Component)]
struct EntityCount;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_fps, update_entity_count));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(0.0),
            right: px(0.0),
            column_gap: px(6.0),
            ..default()
        },
        children![
            (
                Text::new("FPS:"),
                TextFont::from_font_size(FONT_SIZE),
                RenderLayers::layer(HIGH_RES_LAYER),
            ),
            (
                Text::default(),
                TextFont::from_font_size(FONT_SIZE),
                Fps,
                RenderLayers::layer(HIGH_RES_LAYER),
            ),
            (
                Text::new("Entities:"),
                TextFont::from_font_size(FONT_SIZE),
                RenderLayers::layer(HIGH_RES_LAYER),
            ),
            (
                Text::default(),
                TextFont::from_font_size(FONT_SIZE),
                EntityCount,
                RenderLayers::layer(HIGH_RES_LAYER),
            ),
        ],
    ));
}

fn update_fps(mut query: Query<&mut Text, With<Fps>>, diagnostics: Res<DiagnosticsStore>) {
    let mut text = query.single_mut().unwrap();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        && let Some(fps_value) = fps.smoothed()
    {
        **text = format!("{fps_value:.2}");
    }
}

fn update_entity_count(
    mut query: Query<&mut Text, With<EntityCount>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut text = query.single_mut().unwrap();
    if let Some(count) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        && let Some(measurement) = count.measurement()
    {
        let value = measurement.value;
        **text = format!("{value:.2}");
    }
}
