use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::hud;
use crate::hud::HudSubject;
use crate::vessel::Vessel;

pub(super) struct ThrottlePlugin;

impl Plugin for ThrottlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
pub struct ThrottleBar;

#[derive(Component)]
struct ThrottleText;

fn setup(mut commands: Commands) {
    use hud::TextFontExt;
    commands.spawn((
        Name::new("Throttle Widget"),
        Node {
            left: px(20.0),
            bottom: px(20.0),
            height: px(140.0),
            width: px(30.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::FlexEnd,
            border: hud::BORDER,
            border_radius: BorderRadius::all(px(3.0)),
            padding: UiRect::all(px(5.0)),
            ..default()
        },
        hud::BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
        children![
            (
                Node {
                    height: Val::Percent(0.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                ThrottleBar,
                BackgroundColor::from(Color::srgb(0.0, 0.8, 0.32)),
            ),
            (Text::default(), ThrottleText, TextFont::ui_default()),
        ],
    ));
}

fn update(
    vessel: Single<&Vessel, With<HudSubject>>,
    mut throttle_bar_node: Single<&mut Node, With<ThrottleBar>>,
    mut throttle_text: Single<&mut Text, With<ThrottleText>>,
) {
    throttle_bar_node.height = Val::Percent(vessel.throttle * 100.0);
    throttle_text.0 = format!("{:.0}", vessel.throttle * 100.0);
}
