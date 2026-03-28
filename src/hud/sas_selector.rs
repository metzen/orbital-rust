use bevy::color::palettes::css::BLACK;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

use crate::vessel::{Direction, Vessel};

use super::HudSubject;
use super::TextFontExt;

const BG_COLOR: Srgba = Srgba::new(0.0, 0.1, 0.0, 1.0);
const BG_COLOR_SELECTED: Srgba = Srgba::new(0.0, 0.9, 0.0, 1.0);
const BG_COLOR_DISABLED: Srgba = Srgba::new(0.0, 0.02, 0.0, 1.0);
const TEXT_COLOR: Srgba = Srgba::new(0.0, 0.3, 0.0, 1.0);
const TEXT_COLOR_SELECTED: Srgba = Srgba::new(0.0, 0.0, 0.0, 1.0);
const TEXT_COLOR_DISABLED: Srgba = Srgba::new(0.0, 0.15, 0.0, 1.0);

#[derive(Component)]
pub(super) struct StabilizeWidget;

#[derive(Component)]
pub(super) struct ProgradeWidget;

#[derive(Component)]
pub(super) struct RetrogradeWidget;

#[derive(Component)]
pub(super) struct RadialWidget;

#[derive(Component)]
pub(super) struct AntiRadialWidget;

fn widget(component: impl Component, label: impl Into<String>) -> impl Bundle {
    (
        component,
        Node {
            margin: UiRect::vertical(px(4.0)),
            padding: UiRect::horizontal(px(4.0)),
            ..default()
        },
        Text::new(label),
        TextFont::ui_default().with_font_size(12.0),
        TextLayout::new_with_justify(Justify::Center),
        TextColor::from(TEXT_COLOR),
        BackgroundColor::from(BG_COLOR),
    )
}

pub(super) fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("SAS Controls"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(545.0),
            bottom: Val::Px(20.0),
            border: super::BORDER,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        super::BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        children![
            widget(StabilizeWidget, "Stabilize"),
            widget(ProgradeWidget, "Prograde"),
            widget(RetrogradeWidget, "Retrograde"),
            widget(RadialWidget, "Radial-out"),
            widget(AntiRadialWidget, "Radial-in"),
        ],
    ));
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct Colors {
    background: Write<BackgroundColor>,
    text: Write<TextColor>,
}

pub(super) fn update(
    mut set: ParamSet<(
        Single<Colors, With<ProgradeWidget>>,
        Single<Colors, With<RetrogradeWidget>>,
        Single<Colors, With<RadialWidget>>,
        Single<Colors, With<AntiRadialWidget>>,
        Single<Colors, With<StabilizeWidget>>,
    )>,
    vessel: Single<&Vessel, With<HudSubject>>,
) {
    // TODO: Refactor duplicated code.
    // TODO: Dim all when SAS disabled.
    set.p0().background.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::Prograde => BG_COLOR_SELECTED,
                _ => BG_COLOR,
            },
            None => BG_COLOR,
        }
    } else {
        BG_COLOR_DISABLED
    });
    set.p0().text.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::Prograde => TEXT_COLOR_SELECTED,
                _ => TEXT_COLOR,
            },
            None => TEXT_COLOR,
        }
    } else {
        TEXT_COLOR_DISABLED
    });

    set.p1().background.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::Retrograde => BG_COLOR_SELECTED,
                _ => BG_COLOR,
            },
            None => BG_COLOR,
        }
    } else {
        BG_COLOR_DISABLED
    });
    set.p1().text.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::Retrograde => TEXT_COLOR_SELECTED,
                _ => TEXT_COLOR,
            },
            None => TEXT_COLOR,
        }
    } else {
        TEXT_COLOR_DISABLED
    });

    set.p2().background.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::Radial => BG_COLOR_SELECTED,
                _ => BG_COLOR,
            },
            None => BG_COLOR,
        }
    } else {
        BG_COLOR_DISABLED
    });
    set.p2().text.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::Radial => TEXT_COLOR_SELECTED,
                _ => TEXT_COLOR,
            },
            None => TEXT_COLOR,
        }
    } else {
        TEXT_COLOR_DISABLED
    });

    set.p3().background.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::AntiRadial => BG_COLOR_SELECTED,
                _ => BG_COLOR,
            },
            None => BG_COLOR,
        }
    } else {
        BG_COLOR_DISABLED
    });
    set.p3().text.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref direction) => match direction {
                Direction::AntiRadial => TEXT_COLOR_SELECTED,
                _ => TEXT_COLOR,
            },
            None => TEXT_COLOR,
        }
    } else {
        TEXT_COLOR_DISABLED
    });

    set.p4().background.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref _direction) => BG_COLOR,
            None => BG_COLOR_SELECTED,
        }
    } else {
        BG_COLOR_DISABLED
    });
    set.p4().text.0 = Color::from(if vessel.sas_enabled {
        match vessel.direction_lock {
            Some(ref _direction) => TEXT_COLOR,
            None => TEXT_COLOR_SELECTED,
        }
    } else {
        TEXT_COLOR_DISABLED
    });
}
