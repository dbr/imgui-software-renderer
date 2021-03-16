use imgui::im_str;
use imgui::internal::RawWrapper;
use imgui::{DrawCmd, DrawCmdParams};

fn rasterize(width: u32, height: u32) {
    println!("Hm");
    let mut imgui_ctx = imgui::Context::create();
    let font_atlas = imgui_ctx.fonts().build_rgba32_texture();
    println!("2");

    imgui_ctx.io_mut().display_size = [width as f32, height as f32];
    let ui = imgui_ctx.frame();
    ui.button(imgui::im_str!("Huh"), [0.0, 0.0]);
    imgui::Window::new(im_str!("Test"))
    .size([200.0, 100.0], imgui::Condition::FirstUseEver)
    .build(&ui, || {
        ui.button(imgui::im_str!("Hi"), [0.0, 0.0]);
        ui.text("Ok");
    });
    println!("3");

    let draw_data = ui.render();
    dbg!(draw_data.total_vtx_count);
    dbg!(draw_data.total_idx_count);
    dbg!(draw_data.draw_lists_count());

    println!("4");

    use tiny_skia::{Paint, Pixmap, Rect, Transform};
    let mut px = Pixmap::new(width, height).unwrap();
    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 70, 200, 255);

    for draw_list in draw_data.draw_lists() {
        for cmd in draw_list.commands() {
            let idx_buffer = draw_list.idx_buffer();

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
                    dbg!(clip_rect);
                    dbg!(texture_id);
                    dbg!(idx_buffer);

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
    rasterize(1024, 512);
}
