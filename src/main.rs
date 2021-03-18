use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapRef, Transform};

use imgui::internal::RawWrapper;
use imgui::{im_str, FontConfig, FontSource};
use imgui::{DrawCmd, DrawCmdParams};

mod copypaste {
    use tiny_skia::Transform;

    fn dcross(a: f64, b: f64, c: f64, d: f64) -> f64 {
        a * b - c * d
    }

    fn dcross_dscale(a: f32, b: f32, c: f32, d: f32, scale: f64) -> f32 {
        (dcross(a as f64, b as f64, c as f64, d as f64) * scale) as f32
    }

    pub fn compute_inv(ts: &Transform, inv_det: f64) -> Transform {
        Transform::from_row(
            (ts.sy as f64 * inv_det) as f32,
            (-ts.ky as f64 * inv_det) as f32,
            (-ts.kx as f64 * inv_det) as f32,
            (ts.sx as f64 * inv_det) as f32,
            dcross_dscale(ts.kx, ts.ty, ts.sy, ts.tx, inv_det),
            dcross_dscale(ts.ky, ts.tx, ts.sx, ts.ty, inv_det),
        )
    }

    fn is_nearly_zero_within_tolerance(value: f32, tolerance: f32) -> bool {
        debug_assert!(tolerance >= 0.0);
        value.abs() <= tolerance
    }

    fn inv_determinant(ts: &Transform) -> Option<f64> {
        let det = dcross(ts.sx as f64, ts.sy as f64, ts.kx as f64, ts.ky as f64);

        // Since the determinant is on the order of the cube of the matrix members,
        // compare to the cube of the default nearly-zero constant (although an
        // estimate of the condition number would be better if it wasn't so expensive).
        const SCALAR_NEARLY_ZERO: f32 = 1.0 / (1 << 12) as f32;

        let tolerance = SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO;
        if is_nearly_zero_within_tolerance(det as f32, tolerance) {
            None
        } else {
            Some(1.0 / det)
        }
    }

    fn is_finite(x: &Transform) -> bool {
        x.sx.is_finite()
            && x.ky.is_finite()
            && x.kx.is_finite()
            && x.sy.is_finite()
            && x.tx.is_finite()
            && x.ty.is_finite()
    }

    pub fn invert(ts: &Transform) -> Option<Transform> {
        debug_assert!(!ts.is_identity());

        if ts.is_scale_translate() {
            if ts.has_scale() {
                let inv_x = 1.0 / ts.sx;
                let inv_y = 1.0 / ts.sy;
                Some(Transform::from_row(
                    inv_x,
                    0.0,
                    0.0,
                    inv_y,
                    -ts.tx * inv_x,
                    -ts.ty * inv_y,
                ))
            } else {
                // translate only
                Some(Transform::from_translate(-ts.tx, -ts.ty))
            }
        } else {
            let inv_det = inv_determinant(ts)?;
            let inv_ts = compute_inv(ts, inv_det);

            if is_finite(&inv_ts) {
                Some(inv_ts)
            } else {
                None
            }
        }
    }
}

enum DrawPass {
    Fill,
    Texture,
}

fn rasterize(mut px: &mut Pixmap, draw_data: &imgui::DrawData, font_pixmap: PixmapRef) {
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color_rgba8(50, 70, 200, 255);

    for draw_list in draw_data.draw_lists() {
        let verts = draw_list.vtx_buffer();
        for draw_pass in [DrawPass::Fill, DrawPass::Texture].iter() {
            for cmd in draw_list.commands() {
                let idx_buffer = draw_list.idx_buffer();

                match cmd {
                    DrawCmd::Elements {
                        count: _count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect: _clip_rect,
                                texture_id: _texture_id,
                                vtx_offset: _vtx_offset,
                                idx_offset: _idx_offset,
                                ..
                            },
                    } => {
                        // dbg!(count);
                        // dbg!(clip_rect);
                        // dbg!(texture_id);
                        // dbg!(idx_buffer);
                        dbg!(_texture_id);

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

                            match draw_pass {
                                DrawPass::Fill => {
                                    // Paint triangle with color
                                    // FIXME: Only using value from one vertex
                                    let hm = v0;
                                    paint.set_color_rgba8(
                                        hm.col[0], hm.col[1], hm.col[2], hm.col[3],
                                    );
                                    px.fill_path(
                                        &path,
                                        &paint,
                                        tiny_skia::FillRule::default(),
                                        Transform::identity(),
                                        None,
                                    );
                                }

                                DrawPass::Texture => {
                                    // Paint texture
                                    render_textured_tri(
                                        font_pixmap,
                                        v0.uv,
                                        v1.uv,
                                        v2.uv,
                                        &mut px,
                                        v0.pos,
                                        v1.pos,
                                        v2.pos,
                                        None,
                                    );
                                }
                            }

                            // Debug: show poly outline
                            if false {
                                if let DrawPass::Fill = draw_pass {
                                    paint.set_color_rgba8(255, 255, 0, 128);
                                    px.stroke_path(
                                        &path,
                                        &paint,
                                        &tiny_skia::Stroke::default(),
                                        Transform::default(),
                                        None,
                                    );
                                }
                            }
                        }
                    }
                    DrawCmd::ResetRenderState => (), // TODO
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd)
                    },
                }
            }
        }
    }
}

fn cornerpin(ul: (f32, f32), ur: (f32, f32), ll: (f32, f32)) -> Transform {
    // Affine (3 points, no skewing)
    let m11 = ur.0 - ul.0;
    let m12 = ur.1 - ul.1;
    let m21 = ll.0 - ul.0;
    let m22 = ll.1 - ul.1;
    // let m33 = 1;
    let m41 = ul.0;
    let m42 = ul.1;
    // let m44 = 1.0;

    let affine = Transform::from_row(m11, m12, m21, m22, m41, m42);

    affine
}

fn render_textured_tri(
    texture_px: PixmapRef,
    uv_p0: [f32; 2],
    uv_p1: [f32; 2],
    uv_p2: [f32; 2],
    output: &mut Pixmap,
    dest_p0: [f32; 2],
    dest_p1: [f32; 2],
    dest_p2: [f32; 2],
    clip_mask: Option<&tiny_skia::ClipMask>,
) {
    fn p(x: [f32; 2]) -> (f32, f32) {
        (x[0], x[1])
    }
    let xform_image_to_norm = Transform::from_scale(
        1.0 / texture_px.width() as f32,
        1.0 / texture_px.height() as f32,
    );

    let xform = xform_image_to_norm
        .post_concat(copypaste::invert(&cornerpin(p(uv_p0), p(uv_p1), p(uv_p2))).unwrap())
        .post_concat(cornerpin(p(dest_p0), p(dest_p1), p(dest_p2)));

    let tex = tiny_skia::Pattern::new(
        texture_px,
        tiny_skia::SpreadMode::Pad,
        tiny_skia::FilterQuality::Bilinear,
        1.0,
        xform,
    );

    let mut paint = Paint::default();
    paint.shader = tex;

    let mut path = PathBuilder::new();
    path.move_to(dest_p0[0], dest_p0[1]);
    path.line_to(dest_p1[0], dest_p1[1]);
    path.line_to(dest_p2[0], dest_p2[1]);
    path.close();
    let path = path.finish().unwrap();

    output.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::default(),
        Transform::identity(),
        clip_mask,
    );
}

fn test_texture() {
    let texture_px = Pixmap::load_png("uvtest.png").unwrap();
    let mut px = Pixmap::new(512, 512).unwrap();
    render_textured_tri(
        texture_px.as_ref(),
        [0.0, 0.0],
        [1.0, 0.0],
        [0.5, 1.0],
        &mut px,
        [0.0, 0.0],
        [0.0, 500.0],
        [500.0, 50.0],
        None,
    );
    px.save_png("texture_test.png").unwrap();
}

fn main() {
    let width = 1300;
    let height = 700;

    let mut imgui_ctx = imgui::Context::create();
    imgui_ctx.set_ini_filename(None); // Don't want to save window layout
    imgui_ctx.style_mut().use_dark_colors();
    imgui_ctx.style_mut().window_rounding = 5.0;
    imgui_ctx.io_mut().font_global_scale = 2.0;
    imgui_ctx.style_mut().anti_aliased_lines = true;
    imgui_ctx.style_mut().anti_aliased_fill = true;
    imgui_ctx.style_mut().anti_aliased_lines_use_tex = false;
    imgui_ctx.style_mut().scale_all_sizes(2.0);

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

        font_pixmap
    };
    font_pixmap.save_png("font.png").unwrap();

    imgui_ctx.io_mut().display_size = [width as f32, height as f32];
    imgui_ctx.io_mut().display_framebuffer_scale = [1.0, 1.0];

    for frame in 0..6 {
        imgui_ctx
            .io_mut()
            .update_delta_time(std::time::Duration::from_millis(10));

        let draw_data: &imgui::DrawData = {
            let ui = imgui_ctx.frame();

            ui.get_window_draw_list()
                .add_rect([10.0, 10.0], [50.0, 50.0], [0.5, 0.0, 1.0])
                .filled(true)
                .rounding(6.0)
                .build();

            ui.get_window_draw_list()
                .add_text([200.0, 200.0], [0.0, 1.0, 0.0, 1.0], "i");

            imgui::Window::new(im_str!("Test"))
                .size([200.0, 100.0], imgui::Condition::FirstUseEver)
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
        px.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 255));

        // Render imgui data
        let start = std::time::Instant::now();
        rasterize(&mut px, draw_data, font_pixmap.as_ref());
        dbg!(start.elapsed());

        // Save output
        px.save_png(format!("test_{}.png", frame)).unwrap();
        dbg!(start.elapsed());
    }
}
