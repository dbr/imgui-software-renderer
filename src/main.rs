use imgui_software_renderer::drawing;

use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapRef, Transform};

use imgui::internal::RawWrapper;
use imgui::{im_str, FontConfig, FontSource};
use imgui::{DrawCmd, DrawCmdParams};




// fn test_texture() {
//     let texture_px = Pixmap::load_png("uvtest.png").unwrap();
//     let mut px = Pixmap::new(512, 512).unwrap();
//     render_textured_tri(
//         texture_px.as_ref(),
//         [0.0, 0.0],
//         [1.0, 0.0],
//         [0.5, 1.0],
//         &mut px,
//         [0.0, 0.0],
//         [0.0, 500.0],
//         [500.0, 50.0],
//         None,
//     );
//     px.save_png("texture_test.png").unwrap();
// }

fn main() {
    let width = 1200;
    let height = 500;

    let mut imgui_ctx = imgui::Context::create();
    imgui_ctx.set_ini_filename(None); // Don't want to save window layout
    imgui_ctx.style_mut().use_dark_colors();
    imgui_ctx.style_mut().window_rounding = 0.0;
    imgui_ctx.io_mut().font_global_scale = 1.0;
    imgui_ctx.io_mut().mouse_draw_cursor = true;
    imgui_ctx.style_mut().anti_aliased_lines = false;
    imgui_ctx.style_mut().anti_aliased_fill = false;
    imgui_ctx.style_mut().anti_aliased_lines_use_tex = false;
    imgui_ctx.style_mut().scale_all_sizes(1.0);

    imgui_ctx.io_mut().mouse_pos = [200.0, 50.0];

    imgui_ctx.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig {
            size_pixels: 13.0,
            ..FontConfig::default()
        }),
    }]);

    let font_pixmap = {
        let mut font_atlas = imgui_ctx.fonts();
        let font_atlas_tex = font_atlas.build_rgba32_texture();

        let mut font_pixmap = Pixmap::new(font_atlas_tex.width, font_atlas_tex.height).unwrap();

        {
            let data = font_pixmap.pixels_mut();
            for (i, src) in font_atlas_tex.data.chunks(4).enumerate() {
                data[i] =
                    tiny_skia::ColorU8::from_rgba(src[0], src[1], src[2], src[3]).premultiply();
            }
        }

        for (i, mut pixel) in font_pixmap.pixels_mut().iter().enumerate() {
            pixel = &tiny_skia::PremultipliedColorU8::from_rgba(100, 20, 5, 255).unwrap();
        }
        font_pixmap
    };

    imgui_ctx.io_mut().display_size = [width as f32, height as f32];
    imgui_ctx.io_mut().display_framebuffer_scale = [1.0, 1.0];

    for frame in 0..10 {
        println!("Frame {}", frame);
        imgui_ctx
            .io_mut()
            .update_delta_time(std::time::Duration::from_millis(20));

        let draw_data: &imgui::DrawData = {
            let ui = imgui_ctx.frame();

            imgui::Window::new(im_str!("Abc"))
                .position([30.0, 10.0], imgui::Condition::Always)
                .size([120.0, 100.0], imgui::Condition::Always)
                .build(&ui, || {
                    ui.text("a");

                    ui.get_window_draw_list()
                        .add_rect([10.0, 10.0], [50.0, 50.0], [0.5, 0.0, 1.0])
                        .filled(true)
                        .rounding(6.0)
                        .build();
                });

            imgui::Window::new(im_str!("Test"))
                .size([250.0, 100.0], imgui::Condition::FirstUseEver)
                .position([10.0, 200.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.button(imgui::im_str!("Hi"), [0.0, 0.0]);
                    ui.text("Ok");
                    let mut thing = 0.4;
                    ui.input_float(im_str!("##Test"), &mut thing).build();
                });

            ui.show_demo_window(&mut true);
            ui.show_metrics_window(&mut true);

            // Done
            ui.render()
        };

        // Create empty pixmap
        let mut px = Pixmap::new(width, height).unwrap();
        px.fill(tiny_skia::Color::from_rgba8(89, 89, 89, 255));

        // Render imgui data
        let start = std::time::Instant::now();
        let r = imgui_software_renderer::Renderer::new();
        r.render(&mut px, draw_data, font_pixmap.as_ref());
        dbg!(start.elapsed());

        // Save output
        px.save_png(format!("test_{}.png", frame)).unwrap();
        dbg!(start.elapsed());
    }
}
