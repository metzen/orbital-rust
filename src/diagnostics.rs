use bevy::camera::visibility::RenderLayers;
use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;

use crate::camera::HIGH_RES_LAYER;

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
    let text_style = TextFont {
        font_size: 11.0,
        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
        ..default()
    };
    commands
        .spawn(
            Node {
                // fill the entire window
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                // padding: UiRect::all(MARGIN),
                // row_gap: Val::Px(),
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(50.0), Val::Px(0.0)),
                ..default()
            },
            // background_color: BackgroundColor(Color::BLACK),
        )
        .with_children(|root| {
            root.spawn((
                Text::new("FPS: "),
                text_style.clone(),
                RenderLayers::layer(HIGH_RES_LAYER),
            ));
            root.spawn((
                Text::default(),
                text_style.clone(),
                Fps,
                RenderLayers::layer(HIGH_RES_LAYER),
            ));
            root.spawn((
                Text::new("Entities: "),
                text_style.clone(),
                RenderLayers::layer(HIGH_RES_LAYER),
            ));
            root.spawn((
                Text::default(),
                text_style.clone(),
                EntityCount,
                RenderLayers::layer(HIGH_RES_LAYER),
            ));
        });
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
