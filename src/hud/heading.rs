use std::f32::consts::TAU;

use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::text::LineHeight;

use crate::hud::{BORDER, BORDER_COLOR, HudSubject};

pub(super) struct HeadingPlugin;

impl Plugin for HeadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
struct HeadingText;

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Heading widget"),
        Node {
            position_type: PositionType::Absolute,
            left: px(156.0 + 92.0 - 27.0),
            bottom: px(270.0),
            border: BORDER,
            border_radius: BorderRadius::all(px(3.0)),
            padding: UiRect::axes(px(10.0), px(6.0)),
            justify_content: JustifyContent::End,
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
        children![(
            HeadingText,
            Text::default(),
            TextFont::from_font_size(18.0),
            LineHeight::RelativeToFont(1.0),
        )],
    ));
}

fn update(
    mut text: Single<&mut Text, With<HeadingText>>,
    transform: Single<&Transform, With<HudSubject>>,
) {
    // TODO: Determine the real compass heading.
    let (axis, angle) = transform.rotation.to_axis_angle();
    let modifier = if axis.z < 0.0 { TAU } else { 0.0 };
    text.0 = format!("{:3.0}", (modifier + axis.z * angle).to_degrees());
}
