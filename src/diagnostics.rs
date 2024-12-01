use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::camera::HIGH_RES_LAYERS;

#[derive(Component)]
struct Fps;

#[derive(Component)]
struct EntityCount;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin))
            .add_systems(Startup, setup)
            .add_systems(Update, (update_fps, update_entity_count));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Roboto-Medium.ttf");
    let text_style = TextFont {
        font: font.clone(),
        font_size: 11.0,
        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
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
            root.spawn((Text::new("FPS: "), text_style.clone(), HIGH_RES_LAYERS));
            root.spawn((Text::default(), text_style.clone(), Fps, HIGH_RES_LAYERS));
            root.spawn((Text::new("Entities: "), text_style.clone(), HIGH_RES_LAYERS));
            root.spawn((
                Text::default(),
                text_style.clone(),
                EntityCount,
                HIGH_RES_LAYERS,
            ));
        });
}

fn update_fps(mut query: Query<&mut Text, With<Fps>>, diagnostics: Res<DiagnosticsStore>) {
    let mut text = query.single_mut();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps.smoothed() {
            text.0 = format!("{fps_value:.2}");
        }
    }
}

fn update_entity_count(
    mut query: Query<&mut Text, With<EntityCount>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut text = query.single_mut();
    if let Some(count) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT) {
        if let Some(measurement) = count.measurement() {
            let value = measurement.value;
            text.0 = format!("{value:.2}");
        }
    }
}
