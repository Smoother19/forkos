#![allow(dead_code)]

use iced::Color;

pub const BASE: Color = rgb(0xfa, 0xf4, 0xed);
pub const SURFACE: Color = rgb(0xff, 0xfa, 0xf3);
pub const OVERLAY: Color = rgb(0xf2, 0xe9, 0xe1);
pub const MUTED: Color = rgb(0x9d, 0x99, 0xa3);
pub const SUBTLE: Color = rgb(0x79, 0x76, 0x7e);
pub const TEXT: Color = rgb(0x57, 0x52, 0x79);
pub const LOVE: Color = rgb(0xb4, 0x63, 0x7a);
pub const GOLD: Color = rgb(0xea, 0x9d, 0x34);
pub const ROSE: Color = rgb(0xd7, 0x82, 0x7e);
pub const PINE: Color = rgb(0x28, 0x65, 0x7b);
pub const FOAM: Color = rgb(0x56, 0x94, 0x9f);
pub const IRIS: Color = rgb(0x90, 0x7a, 0xa9);
pub const HIGHLIGHT_LOW: Color = rgb(0xf4, 0xed, 0xe8);
pub const HIGHLIGHT_MED: Color = rgb(0xdf, 0xda, 0xd9);

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub fn with_alpha(c: Color, alpha: f32) -> Color {
    Color { r: c.r, g: c.g, b: c.b, a: alpha }
}
