use std::borrow::BorrowMut;

use imgui::internal::RawWrapper;
use imgui::{im_str, FontConfig, FontSource};
use imgui::{DrawCmd, DrawCmdParams};

fn rasterize(width: u32, height: u32) {
    let mut imgui_ctx = imgui::Context::create();
    imgui_ctx.set_ini_filename(None); // Don't want to save window layout
    imgui_ctx.style_mut().use_classic_colors();

    imgui_ctx.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig {
            size_pixels: 13.0,
            ..FontConfig::default()
        }),
    }]);
    let font_atlas = imgui_ctx.fonts().build_rgba32_texture();

    imgui_ctx.io_mut().display_size = [width as f32, height as f32];
    imgui_ctx.io_mut().display_framebuffer_scale = [1.0, 1.0];

    let mut draw_data: &imgui::DrawData = {
        let ui = imgui_ctx.frame();

        ui.get_window_draw_list()
            .add_rect([10.0, 10.0], [50.0, 50.0], [0.5, 0.0, 1.0])
            .build();
        ui.button(imgui::im_str!("Huh"), [0.0, 0.0]);
        imgui::Window::new(im_str!("Test"))
            .size([200.0, 100.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.button(imgui::im_str!("Hi"), [0.0, 0.0]);
                ui.text("Ok");
            });

        // Done
        ui.render()
    };

    // dbg!(draw_data.display_pos);
    // dbg!(draw_data.display_size);
    // dbg!(draw_data.total_vtx_count);
    // dbg!(draw_data.total_idx_count);
    // dbg!(draw_data.draw_lists_count());

    use tiny_skia::{Paint, Pixmap, Rect, Transform};
    let mut px = Pixmap::new(width, height).unwrap();
    px.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 255));

    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color_rgba8(50, 70, 200, 255);

    for draw_list in draw_data.draw_lists() {
        let verts = draw_list.vtx_buffer();
        for cmd in draw_list.commands() {
            let idx_buffer = draw_list.idx_buffer();

            // println!("\n\n\n\nAbout to draw an element");
            match cmd {
                DrawCmd::Elements {
                    count,
                    cmd_params:
                        DrawCmdParams {
                            clip_rect,
                            texture_id,
                            vtx_offset,
                            idx_offset,
                            ..
                        },
                } => {
                    // dbg!(count);
                    // dbg!(clip_rect);
                    // dbg!(texture_id);
                    // dbg!(idx_buffer);

                    for x in idx_buffer.chunks(3) {
                        let v0 = verts[x[0] as usize];
                        let v1 = verts[x[1] as usize];
                        let v2 = verts[x[2] as usize];
                        let path = {
                            let mut pb = tiny_skia::PathBuilder::new();
                            pb.move_to(v0.pos[0], v0.pos[1]);
                            pb.line_to(v1.pos[0], v1.pos[1]);
                            pb.line_to(v2.pos[0], v2.pos[1]);
                            pb.close();
                            pb.finish().unwrap()
                        };
                        paint.set_color_rgba8(v0.col[0], v0.col[1], v0.col[1], v0.col[2]);
                        px.fill_path(
                            &path,
                            &paint,
                            tiny_skia::FillRule::default(),
                            Transform::identity(),
                            None,
                        )
                        .unwrap();
                    }
                    px.fill_rect(
                        Rect::from_xywh(10.0, 10.0, 40.0, 40.0).unwrap(),
                        &paint,
                        Transform::identity(),
                        None,
                    );
                }
                DrawCmd::ResetRenderState => (), // TODO
                DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                    callback(draw_list.raw(), raw_cmd)
                },
            }
        }
    }

    px.save_png("test.png").unwrap();
}

fn main() {
    rasterize(1024, 1024);
}
