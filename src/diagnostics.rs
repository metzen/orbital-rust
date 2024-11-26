use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::camera::HIGH_RES_LAYERS;

#[derive(Component)]
pub struct Fps;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup_fps)
            .add_systems(Update, update_fps);
    }
}

fn setup_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Roboto-Medium.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 10.0,
        color: Color::WHITE,
    };
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("FPS: ", text_style.clone()),
            TextSection::from_style(text_style),
        ]),
        Fps,
        HIGH_RES_LAYERS,
    ));
}

fn update_fps(mut query: Query<&mut Text, With<Fps>>, diagnostics: Res<DiagnosticsStore>) {
    let mut text = query.single_mut();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps.smoothed() {
            text.sections[1].value = format!("{fps_value:.2}");
        }
    }
}
