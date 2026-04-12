use bevy::color::palettes::css::BLACK;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

use super::{HudSubject, TextFontExt};
use crate::vessel::{Direction, Vessel};

const BG_COLOR: Color = Color::srgb(0.439, 0.451, 0.525);
const BG_COLOR_SELECTED: Color = Color::srgb(0.027, 0.69, 0.286);
const BG_COLOR_DISABLED: Color = Color::srgb(0.14, 0.13, 0.16);
const TEXT_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const TEXT_COLOR_SELECTED: Color = Color::srgb(0.0, 0.0, 0.0);
const TEXT_COLOR_DISABLED: Color = Color::srgb(0.0, 0.0, 0.0);

pub struct SasSelectorPlugin;

impl Plugin for SasSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update);
    }
}

#[derive(Component)]
struct StabilizeWidget;

#[derive(Component)]
struct ProgradeWidget;

#[derive(Component)]
struct RetrogradeWidget;

#[derive(Component)]
struct RadialWidget;

#[derive(Component)]
struct AntiRadialWidget;

fn widget(marker: impl Component, label: impl Into<String>) -> impl Bundle {
    (
        marker,
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

fn setup(mut commands: Commands) {
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
struct Colors {
    background: Write<BackgroundColor>,
    text: Write<TextColor>,
}

#[allow(clippy::type_complexity)]
fn update(
    mut set: ParamSet<(
        Single<Colors, With<ProgradeWidget>>,
        Single<Colors, With<RetrogradeWidget>>,
        Single<Colors, With<RadialWidget>>,
        Single<Colors, With<AntiRadialWidget>>,
        Single<Colors, With<StabilizeWidget>>,
    )>,
    vessel: Single<&Vessel, With<HudSubject>>,
) {
    if vessel.sas_enabled {
        set.p0().text.0 = TEXT_COLOR;
        set.p1().text.0 = TEXT_COLOR;
        set.p2().text.0 = TEXT_COLOR;
        set.p3().text.0 = TEXT_COLOR;
        set.p4().text.0 = TEXT_COLOR;
        set.p0().background.0 = BG_COLOR;
        set.p1().background.0 = BG_COLOR;
        set.p2().background.0 = BG_COLOR;
        set.p3().background.0 = BG_COLOR;
        set.p4().background.0 = BG_COLOR;
        match vessel.direction_lock {
            Some(Direction::Prograde) => {
                set.p0().text.0 = TEXT_COLOR_SELECTED;
                set.p0().background.0 = BG_COLOR_SELECTED;
            }
            Some(Direction::Retrograde) => {
                set.p1().text.0 = TEXT_COLOR_SELECTED;
                set.p1().background.0 = BG_COLOR_SELECTED;
            }
            Some(Direction::Radial) => {
                set.p2().text.0 = TEXT_COLOR_SELECTED;
                set.p2().background.0 = BG_COLOR_SELECTED;
            }
            Some(Direction::AntiRadial) => {
                set.p3().text.0 = TEXT_COLOR_SELECTED;
                set.p3().background.0 = BG_COLOR_SELECTED;
            }
            None => {
                set.p4().text.0 = TEXT_COLOR_SELECTED;
                set.p4().background.0 = BG_COLOR_SELECTED;
            }
        }
    } else {
        set.p0().text.0 = TEXT_COLOR_DISABLED;
        set.p1().text.0 = TEXT_COLOR_DISABLED;
        set.p2().text.0 = TEXT_COLOR_DISABLED;
        set.p3().text.0 = TEXT_COLOR_DISABLED;
        set.p4().text.0 = TEXT_COLOR_DISABLED;
        set.p0().background.0 = BG_COLOR_DISABLED;
        set.p1().background.0 = BG_COLOR_DISABLED;
        set.p2().background.0 = BG_COLOR_DISABLED;
        set.p3().background.0 = BG_COLOR_DISABLED;
        set.p4().background.0 = BG_COLOR_DISABLED;
    }
}
