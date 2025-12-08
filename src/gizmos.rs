//! Additional [`GizmoBuffer`] Functions
//!
//! Includes the implementation of [`GizmoBuffer::hyperbola`] and [`GizmoBuffer::hyperbola_2d`],
//! and assorted support items.

use core::f32::consts::TAU;
use std::f32::consts::{FRAC_PI_2, PI};
use std::iter::zip;

use bevy::color::Color;
use bevy::gizmos::config::GizmoConfigGroup;
use bevy::gizmos::gizmos::{GizmoBuffer, Gizmos};
use bevy::math::{Isometry2d, Isometry3d, Vec2, ops};

pub(crate) const DEFAULT_HYPERBOLA_RESOLUTION: u32 = 32;

fn hyperbola_inner(half_size: Vec2, resolution: u32) -> impl Iterator<Item = Vec2> {
    let half_resolution = (resolution / 2) as i32;
    (-half_resolution..=half_resolution).map(move |i| {
        let angle = i as f32 * TAU / resolution as f32;
        Vec2::new(ops::cosh(angle), ops::sinh(angle)) * half_size
    })
}

fn ellipse_inner(start_angle: f32, half_size: Vec2, resolution: u32) -> impl Iterator<Item = Vec2> {
    (0..resolution + 1).map(move |i| {
        let angle = -(i as f32 * TAU / resolution as f32);
        let (x, y) = ops::sin_cos(-start_angle + angle + PI);
        Vec2::new(x, y) * half_size
    })
}

/// A builder returned by [`GizmosExt::hyperbola`].
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

/// A builder returned by [`GizmosExt::hyperbola_2d`].
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

/// A builder returned by [`GizmosExt::ellipse_gradient_2d`].
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
        let colors = (0..self.resolution + 1).map(|i| {
            let factor = i as f32 / self.resolution as f32;
            self.start_color.mix(&self.end_color, factor)
        });
        self.gizmos.linestrip_gradient_2d(zip(positions, colors));
    }
}

/// Additional shape drawing extensions for [`Gizmos`].
pub trait GizmosExt<'w, 's, Config, Clear>
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
    ) -> EllipseGradient2dBuilder<'_, Config, Clear>;

    fn hyperbola(
        &mut self,
        isometry: impl Into<Isometry3d>,
        half_size: Vec2,
        color: impl Into<Color>,
    ) -> HyperbolaBuilder<'_, Config, Clear>;

    fn hyperbola_2d(
        &mut self,
        isometry: impl Into<Isometry2d>,
        half_size: Vec2,
        color: impl Into<Color>,
    ) -> Hyperbola2dBuilder<'_, Config, Clear>;
}

impl<'w, 's, Config, Clear> GizmosExt<'w, 's, Config, Clear> for Gizmos<'w, 's, Config, Clear>
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
        use std::ops::DerefMut;
        EllipseGradient2dBuilder {
            gizmos: self.deref_mut(),
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
        use std::ops::DerefMut;
        HyperbolaBuilder {
            gizmos: self.deref_mut(),
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
        use std::ops::DerefMut;
        Hyperbola2dBuilder {
            gizmos: self.deref_mut(),
            isometry: isometry.into(),
            half_size,
            color: color.into(),
            resolution: DEFAULT_HYPERBOLA_RESOLUTION,
        }
    }
}
