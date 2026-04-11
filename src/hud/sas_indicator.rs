use bevy::prelude::*;

use crate::hud::HudSubject;
use crate::vessel::Vessel;

pub(super) struct SasIndicatorPlugin;

impl Plugin for SasIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
struct SasIndicator;

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(250.0),
            bottom: px(100.0),
            ..default()
        },
        Text::new("SAS"),
        TextFont::ui_default().with_font_size(14.0),
        BackgroundColor::from(Srgba::new(0.0, 0.1, 0.0, 1.0)),
        TextColor::from(Srgba::new(0.0, 0.9, 0.0, 1.0)),
        SasIndicator,
        ZIndex(2),
    ));
}

fn update(
    mut widget: Single<(&mut BackgroundColor, &mut TextColor), With<SasIndicator>>,
    vessel: Single<&Vessel, With<HudSubject>>,
) {
    widget.0.0 = Color::from(match vessel.sas_enabled {
        true => Srgba::new(0.0, 0.9, 0.0, 1.0),
        false => Srgba::new(0.0, 0.1, 0.0, 1.0),
    });
    widget.1.0 = Color::from(match vessel.sas_enabled {
        true => Srgba::new(0.0, 0.0, 0.0, 1.0),
        false => Srgba::new(0.0, 0.3, 0.0, 1.0),
    });
}
