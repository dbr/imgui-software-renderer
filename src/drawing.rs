use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapRef, Transform};

use imgui::internal::RawWrapper;
use imgui::{im_str, FontConfig, FontSource};
use imgui::{DrawCmd, DrawCmdParams};


/// Transform which takes three corners of a 0..1 cube and maps them
/// to the specified coordinates.
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

/// Render a triangle using the given texture UV coordinates, to the
/// specified destination points. Encodes some imgui specific
/// weirdness about texture lookups.
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
    col_p0: [u8; 4],
    col_p1: [u8; 4],
    col_p2: [u8; 4],
) {

    /// Convert between imgui-rs and tiny_skia point encodings
    fn p(x: [f32; 2]) -> (f32, f32) {
        (x[0], x[1])
    }

    // Calculate the transform for the texture lookups.
    // 1. Transform from image coordinates into 0..1 space
    let xform_image_to_norm = Transform::from_scale(
        1.0 / texture_px.width() as f32,
        1.0 / texture_px.height() as f32,
    );
    let xform = xform_image_to_norm
        // 2. Then use the UV coordinates
        .post_concat(crate::copypaste::invert(&cornerpin(p(uv_p0), p(uv_p1), p(uv_p2))).unwrap())
        // 3. and destination coordinates
        .post_concat(cornerpin(p(dest_p0), p(dest_p1), p(dest_p2)));

    // `Pattern` is tiny_skia's name for image shader
    let tex = tiny_skia::Pattern::new(
        texture_px,
        tiny_skia::SpreadMode::Pad,
        tiny_skia::FilterQuality::Bilinear,
        col_p0[3] as f32 / 255.0,
        xform,
    );

    // Path for the triangle
    let path = {
        let mut path = PathBuilder::new();
        path.move_to(dest_p0[0], dest_p0[1]);
        path.line_to(dest_p1[0], dest_p1[1]);
        path.line_to(dest_p2[0], dest_p2[1]);
        path.close();
        path.finish().unwrap()
    };

    // Check if the tri is a single colour (e.g window background
    // area), or requires a texture lookup (e.g text or icon)
    let is_solid = true
        && uv_p0[0] == uv_p1[0]
        && uv_p1[0] == uv_p2[0]
        && uv_p0[1] == uv_p1[1]
        && uv_p1[1] == uv_p2[1];
    // TODO: Better check

    if is_solid {
        // Draw a non-textured triangle

        let mut base_paint = Paint::default();
        // TODO: Gradient between col_p0/1/2, currently using first colour for entire surface which is wrong
        base_paint.set_color_rgba8(col_p0[0], col_p0[1], col_p0[2], col_p0[3]);

        output.fill_path(
            &path,
            &base_paint,
            tiny_skia::FillRule::default(),
            Transform::identity(),
            clip_mask,
        );
    } else {
        // Draw a textured triangle
        let mut paint = Paint::default();
        paint.shader = tex;

        output.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::default(),
            Transform::identity(),
            clip_mask,
        );
    }
}


pub(crate) fn rasterize(mut px: &mut Pixmap, draw_data: &imgui::DrawData, font_pixmap: PixmapRef) {
    let mut counter = 0;

    let mut paint = Paint::default();
    paint.anti_alias = false;
    paint.set_color_rgba8(50, 70, 200, 255);

    for draw_list in draw_data.draw_lists() {
        let verts = draw_list.vtx_buffer();
        for cmd in draw_list.commands() {
            let idx_buffer = draw_list.idx_buffer();

            match cmd {
                DrawCmd::Elements {
                    count: count,
                    cmd_params:
                        DrawCmdParams {
                            clip_rect: clip_rect,
                            texture_id: _texture_id,
                            vtx_offset: vtx_offset,
                            idx_offset: idx_offset,
                            ..
                        },
                } => {
                    assert!(vtx_offset == 0);

                    // dbg!(count);
                    // dbg!(clip_rect);
                    // dbg!(texture_id);
                    // dbg!(idx_buffer);
                    // dbg!(_texture_id);
                    // dbg!(idx_buffer.chunks(3).len());

                    for x in idx_buffer[idx_offset..].chunks(3) {
                        let v0 = verts[x[0] as usize];
                        let v1 = verts[x[1] as usize];
                        let v2 = verts[x[2] as usize];

                        if false {
                            println!("{}", counter);
                            println!("v0: pos: x: {:4.2} y: {:4.2}.   uv : x: {:4.5} y: {:4.5}.   col: r: {:3.0} g: {:3.0} b: {:3.0} a: {:3.0}", v0.pos[0], v0.pos[1], v0.uv[0], v0.uv[1], v0.col[0], v0.col[1], v0.col[2], v0.col[3]);
                            println!("v0: pos: x: {:4.2} y: {:4.2}.   uv : x: {:4.5} y: {:4.5}.   col: r: {:3.0} g: {:3.0} b: {:3.0} a: {:3.0}", v1.pos[0], v1.pos[1], v1.uv[0], v1.uv[1], v1.col[0], v1.col[1], v1.col[2], v1.col[3]);
                            println!("v0: pos: x: {:4.2} y: {:4.2}.   uv : x: {:4.5} y: {:4.5}.   col: r: {:3.0} g: {:3.0} b: {:3.0} a: {:3.0}", v2.pos[0], v2.pos[1], v2.uv[0], v2.uv[1], v2.col[0], v2.col[1], v2.col[2], v2.col[3]);
                            println!("");
                        }

                        let path = {
                            let mut pb = tiny_skia::PathBuilder::new();
                            pb.move_to(v0.pos[0], v0.pos[1]);
                            pb.line_to(v1.pos[0], v1.pos[1]);
                            pb.line_to(v2.pos[0], v2.pos[1]);
                            pb.close();
                            pb.finish().unwrap()
                        };

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
                            v0.col,
                            v1.col,
                            v2.col,
                        );

                        // Debug: show poly outline
                        if false {
                            paint.set_color_rgba8(255, 255, 0, 128);
                            px.stroke_path(
                                &path,
                                &paint,
                                &tiny_skia::Stroke::default(),
                                Transform::default(),
                                None,
                            );
                        }

                        // px.save_png(format!("debug_{}.png", counter)).unwrap();
                        counter += 1;
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
