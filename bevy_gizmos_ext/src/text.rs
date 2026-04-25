//! This module draws text gizmos using a stroke font.

use bevy::math::{Vec2, vec2};
use core::ops::Range;

use crate::simplex_stroke_font::*;

/// A stroke font containing glyphs for the 95 printable ASCII codes.
pub struct StrokeFont<'a> {
    /// Baseline-to-baseline line height ratio.
    #[expect(unused)]
    pub line_height: f32,
    /// Full glyph height (cap + descender) in font units.
    #[expect(unused)]
    pub height: f32,
    /// Cap height in font units.
    pub cap_height: f32,
    /// Advance used for unsupported glyphs.
    #[expect(unused)]
    pub advance: i8,
    /// Raw glyph point positions.
    pub positions: &'a [[i8; 2]],
    /// Stroke ranges into `positions`.
    pub strokes: &'a [Range<usize>],
    /// Glyph advances and stroke ranges.
    pub glyphs: &'a [(i8, Range<usize>); 95],
}

impl<'a> StrokeFont<'a> {
    /// Builds a `StrokeTextLayout` for `text` at the requested `font_size`.
    pub fn layout(&'a self, text: &'a str, font_size: f32) -> StrokeTextLayout<'a> {
        let scale = font_size / SIMPLEX_CAP_HEIGHT;
        let glyph_height = SIMPLEX_HEIGHT * scale;
        let line_height = LINE_HEIGHT * glyph_height;
        let margin_top = line_height - glyph_height;
        let space_advance = SIMPLEX_GLYPHS[0].0 as f32 * scale;
        StrokeTextLayout {
            font: self,
            scale,
            line_height,
            margin_top,
            space_advance,
            text,
        }
    }

    fn get_glyph_index(&self, c: char) -> Option<usize> {
        let code = c as u32;
        if (0x20..=0x7E).contains(&code) {
            Some(code as usize - 0x20)
        } else {
            None
        }
    }

    /// Get the advance and stroke point ranges for a glyph.
    pub fn get_glyph(&self, c: char) -> Option<(i8, Range<usize>)> {
        Some(self.glyphs[self.get_glyph_index(c)?].clone())
    }

    /// Get the advance for a glyph.
    pub fn get_glyph_advance(&self, c: char) -> Option<i8> {
        Some(self.glyphs[self.get_glyph_index(c)?].0)
    }
}

/// Stroke text layout
pub struct StrokeTextLayout<'a> {
    /// The unscaled font
    font: &'a StrokeFont<'a>,
    /// The text
    text: &'a str,
    /// Scale applied to the raw glyph positions.
    scale: f32,
    /// Height of each line of text.
    line_height: f32,
    /// Space between top of line and cap height.
    margin_top: f32,
    /// Width of a space.
    space_advance: f32,
}

impl<'a> StrokeTextLayout<'a> {
    /// Computes the width and height of a text layout with this font and
    /// the given text.
    ///
    /// Returns the layout size in pixels.
    pub fn measure(&self) -> Vec2 {
        let mut layout_size = vec2(0., self.line_height);

        let mut line_width = 0.;
        for c in self.text.chars() {
            if c == '\n' {
                layout_size.x = layout_size.x.max(line_width);
                line_width = 0.;
                layout_size.y += self.line_height;
                continue;
            }

            line_width += self
                .font
                .get_glyph_advance(c)
                .map(|advance| advance as f32 * self.scale)
                .unwrap_or(self.space_advance);
        }

        layout_size.x = layout_size.x.max(line_width);
        layout_size
    }

    /// Returns an iterator over the font strokes for this text layout,
    /// grouped into polylines of `Vec2` points.
    pub fn render(&'a self) -> impl Iterator<Item = impl Iterator<Item = Vec2>> + 'a {
        let mut chars = self.text.chars();
        let mut x = 0.0;
        let mut y = -self.margin_top;
        let mut current_strokes: Range<usize> = 0..0;
        let mut current_x = 0.0;

        core::iter::from_fn(move || {
            loop {
                if !current_strokes.is_empty() {
                    for stroke_index in current_strokes.by_ref() {
                        let stroke = self.font.strokes[stroke_index].clone();
                        if stroke.len() < 2 {
                            continue;
                        }

                        return Some(stroke.map(move |index| {
                            let [p, q] = self.font.positions[index];
                            Vec2::new(
                                current_x + self.scale * p as f32,
                                y - self.scale * (self.font.cap_height - q as f32),
                            )
                        }));
                    }
                }

                let c = chars.next()?;
                if c == '\n' {
                    x = 0.0;
                    y -= self.line_height;
                    continue;
                }

                let Some((advance, strokes)) = self.font.get_glyph(c) else {
                    x += self.space_advance;
                    continue;
                };
                current_strokes = strokes;
                current_x = x;

                x += advance as f32 * self.scale;
            }
        })
    }
}
