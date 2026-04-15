use bevy::prelude::*;

pub(super) fn attitude_indicator() -> impl Bundle {
    (
        Name::new("Attitude indicator"),
        Node {
            width: px(172.0),
            height: px(172.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ZIndex(2),
        children![
            (
                Node {
                    width: percent(100.0),
                    height: percent(50.0),
                    border_radius: BorderRadius::px(16.0, 16.0, 0.0, 0.0),
                    ..default()
                },
                BackgroundColor::from(Color::srgb(0.03, 0.32, 0.56)),
            ),
            (
                Node {
                    width: percent(100.0),
                    height: percent(50.0),
                    border_radius: BorderRadius::px(0.0, 0.0, 16.0, 16.0),
                    ..default()
                },
                BackgroundColor::from(Color::srgb(0.29, 0.22, 0.09)),
            ),
        ],
    )
}
