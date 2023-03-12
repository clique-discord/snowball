use crate::{Vec2d, SIZE};
use forma_render::cpu::{
    buffer::{layout::LinearLayout, BufferBuilder, BufferLayerCache},
    Renderer, RGBA,
};
use forma_render::math::{AffineTransform, Point};
use forma_render::styling::{Color, Fill, Func, Props, Style};
use forma_render::{Composition, Path, PathBuilder};
use std::io::Write;

pub use forma_render::Order;

const NODE_RADIUS: f32 = 10.;
const BACKGROUND_COLOUR: [u8; 3] = [238, 232, 213];

fn node_path() -> Path {
    let weight = 2.0f32.sqrt() / 2.;
    let mut builder = PathBuilder::new();
    builder.move_to(Point::new(0., NODE_RADIUS));
    builder.rat_quad_to(
        Point::new(NODE_RADIUS, NODE_RADIUS),
        Point::new(NODE_RADIUS, 0.),
        weight,
    );
    builder.rat_quad_to(
        Point::new(NODE_RADIUS, -NODE_RADIUS),
        Point::new(0., -NODE_RADIUS),
        weight,
    );
    builder.rat_quad_to(
        Point::new(-NODE_RADIUS, -NODE_RADIUS),
        Point::new(-NODE_RADIUS, 0.),
        weight,
    );
    builder.rat_quad_to(
        Point::new(-NODE_RADIUS, NODE_RADIUS),
        Point::new(0., NODE_RADIUS),
        weight,
    );
    builder.build()
}

fn colour_from_rgb(rgb: [u8; 3]) -> Color {
    let [r, g, b] = rgb;
    let (r, g, b) = (r as f32 / 255., g as f32 / 255., b as f32 / 255.);
    // Convert sRGB to linear, as forma_render uses a linear colour space and it is cheaper to do
    // the conversion here than on each pixel value.
    let (r, g, b) = (r.powf(2.2), g.powf(2.2), b.powf(2.2));
    Color { r, g, b, a: 1.0 }
}

fn solid_fill(rgb: [u8; 3]) -> Props {
    Props {
        func: Func::Draw(Style {
            fill: Fill::Solid(colour_from_rgb(rgb)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[derive(Debug)]
pub struct Drawing {
    composition: Composition,
    renderer: Renderer,
    cache: BufferLayerCache,
    buffer: Vec<u8>,
    bg_col: Color,
    next_order: u32,
}

impl Default for Drawing {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawing {
    pub fn new() -> Self {
        let mut composition = Composition::new();
        /*
        let mut background = composition.create_layer();
        let mut bg_path = PathBuilder::new();
        bg_path.move_to(Point::new(0., 0.));
        bg_path.line_to(Point::new(SIZE, 0.));
        bg_path.line_to(Point::new(SIZE, SIZE));
        bg_path.line_to(Point::new(0., SIZE));
        bg_path.line_to(Point::new(0., 0.));
        background.insert(&bg_path.build());
        background.set_props(solid_fill(BACKGROUND_COLOUR));
        composition.insert(Order::new(0).unwrap(), background);
        */
        let mut renderer = Renderer::new();
        let cache = renderer.create_buffer_layer_cache().unwrap();
        let size = SIZE as usize;
        let buffer = vec![0; size * size * 4];
        let bg_col = colour_from_rgb(BACKGROUND_COLOUR);
        Self {
            composition,
            renderer,
            cache,
            buffer,
            bg_col,
            next_order: 1,
        }
    }

    pub fn add_node(&mut self, colour: [u8; 3]) -> Order {
        let mut layer = self.composition.create_layer();
        layer.insert(&node_path());
        layer.set_props(solid_fill(colour));
        let order = Order::new(self.next_order).unwrap();
        self.next_order += 1;
        self.composition.insert(order, layer);
        order
    }

    pub fn place_node(&mut self, order: Order, center: Vec2d) {
        let layer = self.composition.get_mut(order).unwrap();
        let transform = AffineTransform {
            // x' = x * 1 + y * 0 + 1 * translate_x
            ux: 1.,
            vx: 0.,
            tx: center.x,
            // y' = x * 0 + y * 1 + 1 * translate_y
            uy: 0.,
            vy: 1.,
            ty: center.y,
        };
        layer.set_transform(transform.try_into().unwrap());
    }

    pub fn hide_node(&mut self, order: Order) {
        let layer = self.composition.get_mut(order).unwrap();
        layer.disable();
    }

    pub fn show_node(&mut self, order: Order) {
        let layer = self.composition.get_mut(order).unwrap();
        layer.enable();
    }

    pub fn render_frame(&mut self) {
        let size = SIZE as usize;
        self.renderer.render(
            &mut self.composition,
            &mut BufferBuilder::new(
                &mut self.buffer,
                &mut LinearLayout::new(size, size * 4, size),
            )
            .layer_cache(self.cache.clone())
            .build(),
            RGBA,
            self.bg_col,
            None,
        );
    }

    #[cfg(feature = "png")]
    pub fn frame_as_png(&self, w: impl Write) {
        let size = SIZE as u32;
        let mut encoder = png::Encoder::new(w, size, size);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&self.buffer).unwrap();
    }

    #[cfg(feature = "gif")]
    pub fn frame_as_gif(&mut self) -> gif::Frame {
        let size = SIZE as u16;
        gif::Frame::from_rgba_speed(size, size, &mut self.buffer, 20)
    }
}
