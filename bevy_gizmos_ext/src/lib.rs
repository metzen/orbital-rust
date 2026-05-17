//! Additional [`GizmoBuffer`] functions
//!
//! Includes the implementation of the following Gizmo extensions and assorted
//! support items:
//!
//!   * [`GizmoBufferExt::ellipse_gradient_2d`]
//!   * [`GizmoBufferExt::hyperbola`]
//!   * [`GizmoBufferExt::hyperbola_2d`]
//!   * [`GizmoBufferExt::text`]
//!   * [`GizmoBufferExt::text_2d`]

#[macro_use]
extern crate extension_traits;

use core::f32::consts::TAU;
use std::f32::consts::{FRAC_PI_2, PI};
use std::iter::zip;

use bevy::camera::visibility::RenderLayers;
use bevy::color::Color;
use bevy::gizmos::config::{GizmoConfig, GizmoConfigGroup, GizmoLineConfig, GizmoLineJoint};
use bevy::gizmos::gizmos::GizmoBuffer;
use bevy::math::{Isometry2d, Isometry3d, Vec2, ops, vec2};

use crate::simplex_stroke_font::SIMPLEX_STROKE_FONT;

mod simplex_stroke_font;
mod text;

pub(crate) const DEFAULT_HYPERBOLA_RESOLUTION: u32 = 32;

fn hyperbola_inner(half_size: Vec2, resolution: u32) -> impl Iterator<Item = Vec2> {
    let half_resolution = i32::try_from(resolution / 2).unwrap();
    (-half_resolution..=half_resolution).map(move |i| {
        let angle = i as f32 * TAU / resolution as f32;
        Vec2::new(ops::cosh(angle), ops::sinh(angle)) * half_size
    })
}

fn ellipse_inner(start_angle: f32, half_size: Vec2, resolution: u32) -> impl Iterator<Item = Vec2> {
    (0..=resolution).map(move |i| {
        let angle = -(i as f32 * TAU / resolution as f32);
        let (x, y) = ops::sin_cos(-start_angle + angle + PI);
        Vec2::new(x, y) * half_size
    })
}

/// A builder returned by [`GizmoBufferExt::hyperbola`].
pub struct HyperbolaBuilder<'a, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    gizmos: &'a mut GizmoBuffer<Config, Clear>,
    isometry: Isometry3d,
    half_size: Vec2,
    color: Color,
    resolution: u32,
}

impl<Config, Clear> HyperbolaBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Set the number of line-segments used to approximate the geometry of this hyperbola.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }
}

impl<Config, Clear> Drop for HyperbolaBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn drop(&mut self) {
        // TODO: Return early when Gizmos are disabled.
        // if !self.gizmos.enabled {
        //     return;
        // }

        let positions = hyperbola_inner(self.half_size, self.resolution)
            .map(|vec2| self.isometry * vec2.extend(0.0));
        self.gizmos.linestrip(positions, self.color);
    }
}

/// A builder returned by [`GizmoBufferExt::hyperbola_2d`].
pub struct Hyperbola2dBuilder<'a, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    gizmos: &'a mut GizmoBuffer<Config, Clear>,
    isometry: Isometry2d,
    half_size: Vec2,
    color: Color,
    resolution: u32,
}

impl<Config, Clear> Hyperbola2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Set the number of line-segments used to approximate the geometry of this hyperbola.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }
}

impl<Config, Clear> Drop for Hyperbola2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn drop(&mut self) {
        // TODO: Return early when Gizmos are disabled.
        // if !self.gizmos.enabled {
        //     return;
        // };

        let positions =
            hyperbola_inner(self.half_size, self.resolution).map(|vec2| self.isometry * vec2);
        self.gizmos.linestrip_2d(positions, self.color);
    }
}

/// A builder returned by [`GizmoBufferExt::ellipse_gradient_2d`].
pub struct EllipseGradient2dBuilder<'a, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    gizmos: &'a mut GizmoBuffer<Config, Clear>,
    isometry: Isometry2d,
    half_size: Vec2,
    start_angle: f32,
    start_color: Color,
    end_color: Color,
    resolution: u32,
}

impl<Config, Clear> EllipseGradient2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Set the number of line-segments used to approximate the geometry of this hyperbola.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }
}

impl<Config, Clear> Drop for EllipseGradient2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn drop(&mut self) {
        use bevy::prelude::Mix;
        // TODO: Return early when Gizmos are disabled.
        // if !self.gizmos.enabled {
        //     return;
        // }
        let positions = ellipse_inner(
            self.start_angle - FRAC_PI_2,
            self.half_size,
            self.resolution,
        )
        .map(|vec2| self.isometry * vec2);
        let colors = (0..=self.resolution).map(|i| {
            let factor = i as f32 / self.resolution as f32;
            self.start_color.mix(&self.end_color, factor)
        });
        self.gizmos.linestrip_gradient_2d(zip(positions, colors));
    }
}

/// Additional shape drawing extensions for [`GizmoBuffer`].
#[extension(pub trait GizmoBufferExt)]
impl<Config, Clear> GizmoBuffer<Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn ellipse_gradient_2d(
        &mut self,
        isometry: impl Into<Isometry2d>,
        half_size: Vec2,
        start_angle: f32,
        start_color: impl Into<Color>,
        end_color: impl Into<Color>,
    ) -> EllipseGradient2dBuilder<'_, Config, Clear> {
        EllipseGradient2dBuilder {
            gizmos: self,
            isometry: isometry.into(),
            half_size,
            start_angle,
            start_color: start_color.into(),
            end_color: end_color.into(),
            resolution: DEFAULT_HYPERBOLA_RESOLUTION,
        }
    }

    fn hyperbola(
        &mut self,
        isometry: impl Into<Isometry3d>,
        half_size: Vec2,
        color: impl Into<Color>,
    ) -> HyperbolaBuilder<'_, Config, Clear> {
        HyperbolaBuilder {
            gizmos: self,
            isometry: isometry.into(),
            half_size,
            color: color.into(),
            resolution: DEFAULT_HYPERBOLA_RESOLUTION,
        }
    }

    fn hyperbola_2d(
        &mut self,
        isometry: impl Into<Isometry2d>,
        half_size: Vec2,
        color: impl Into<Color>,
    ) -> Hyperbola2dBuilder<'_, Config, Clear> {
        Hyperbola2dBuilder {
            gizmos: self,
            isometry: isometry.into(),
            half_size,
            color: color.into(),
            resolution: DEFAULT_HYPERBOLA_RESOLUTION,
        }
    }

    fn radial_2d(
        &mut self,
        isometry: impl Into<Isometry2d>,
        start: f32,
        end: f32,
        color: impl Into<Color>,
    ) {
        let isometry = isometry.into();
        self.line_2d(
            isometry.translation + isometry.rotation * Vec2::Y * start,
            isometry.translation + isometry.rotation * Vec2::Y * end,
            color,
        );
    }

    /// Draw text using a stroke font with the given isometry applied.
    ///
    /// Only ASCII characters in the range 32–126 are supported.
    ///
    /// # Arguments
    ///
    /// - `isometry`: defines the translation and rotation of the text.
    /// - `text`: the text to be drawn.
    /// - `size`: the size of the text in pixels.
    /// - `anchor`: normalized anchor point relative to the text bounds,
    ///   where `(0, 0)` is centered, `(-0.5, 0.5)` is top-left,
    ///   and `(0.5, -0.5)` is bottom-right.
    /// - `color`: the color of the text.
    ///
    /// # Example
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use bevy_color::Color;
    /// fn system(mut gizmos: Gizmos) {
    ///     gizmos.text(Isometry3d::IDENTITY, "text gizmo", 25., Vec2::ZERO, Color::WHITE);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    fn text(
        &mut self,
        isometry: impl Into<Isometry3d>,
        text: &str,
        font_size: f32,
        anchor: Vec2,
        color: impl Into<Color>,
    ) {
        let isometry: Isometry3d = isometry.into();
        let color = color.into();
        let layout = SIMPLEX_STROKE_FONT.layout(text, font_size);
        let layout_anchor = layout.measure() * (vec2(-0.5, 0.5) - anchor);
        for points in layout.render() {
            self.linestrip(
                points.map(|point| isometry * (layout_anchor + point).extend(0.)),
                color,
            );
        }
    }

    /// Draw text using a stroke font in 2d with the given isometry applied.
    ///
    /// Only ASCII characters in the range 32–126 are supported.
    ///
    /// # Arguments
    ///
    /// - `isometry`: defines the translation and rotation of the text.
    /// - `text`: the text to be drawn.
    /// - `size`: the size of the text.
    /// - `anchor`: normalized anchor point relative to the text bounds,
    ///   where `(0., 0.)` is centered, `(-0.5, 0.5)` is top-left,
    ///   and `(0.5, -0.5)` is bottom-right.
    /// - `color`: the color of the text.
    ///
    /// # Example
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use bevy_color::Color;
    /// fn system(mut gizmos: Gizmos) {
    ///     gizmos.text_2d(Isometry2d::IDENTITY, "2D text gizmo", 25., Vec2::ZERO, Color::WHITE);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    fn text_2d(
        &mut self,
        isometry: impl Into<Isometry2d>,
        text: &str,
        font_size: f32,
        anchor: Vec2,
        color: impl Into<Color>,
    ) {
        let isometry: Isometry2d = isometry.into();
        let color = color.into();
        let layout = SIMPLEX_STROKE_FONT.layout(text, font_size);
        let layout_anchor = layout.measure() * (vec2(-0.5, 0.5) - anchor);
        for points in layout.render() {
            self.linestrip_2d(
                points.map(|point| isometry * (layout_anchor + point)),
                color,
            );
        }
    }
}

/// Extension trait with extra utility methods for [`GizmoConfig`].
#[extension(pub trait GizmoConfigExt)]
impl GizmoConfig {
    fn with_render_layers(self, render_layers: RenderLayers) -> Self {
        Self {
            render_layers,
            ..self
        }
    }

    fn with_line(self, line: GizmoLineConfig) -> Self {
        Self { line, ..self }
    }
}

/// Extension trait with extra utility methods for [`GizmoLineConfig`].
#[extension(pub trait GizmoLineConfigExt)]
impl GizmoLineConfig {
    fn with_width(self, width: f32) -> Self {
        Self { width, ..self }
    }
    fn with_joints(self, joints: GizmoLineJoint) -> Self {
        Self { joints, ..self }
    }
}
