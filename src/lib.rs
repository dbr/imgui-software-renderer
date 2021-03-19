pub mod drawing;
mod copypaste;

use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapRef, Transform};

use imgui::internal::RawWrapper;
use imgui::{im_str, FontConfig, FontSource};
use imgui::{DrawCmd, DrawCmdParams};

pub struct Renderer {
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }
    pub fn render(&self, mut px: &mut Pixmap, draw_data: &imgui::DrawData, font_pixmap: PixmapRef) {
        crate::drawing::rasterize(&mut px, draw_data, font_pixmap);

    }
}
