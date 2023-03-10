use crate::{Graph, HasKey, Vec2d, SIZE};
use tiny_skia::{Pixmap, Color, Paint, Path, PathBuilder, Stroke, Transform, FillRule};

const WEIGHT_DISPLAY_THRESHOLD: f32 = 1.;
const WEIGHT_COLOUR_START: f32 = 5.;
const WEIGHT_COLOUR_END: f32 = 15.;
const NODE_RADIUS: f32 = 10.;
const EDGE_WIDTH: f32 = 1.;
const BACKGROUND_COLOUR: [u8; 3] = [238, 232, 213];

fn weight_colour(mut weight: f32) -> Color {
    weight = weight.log2().clamp(WEIGHT_COLOUR_START, WEIGHT_COLOUR_END);
    weight -= WEIGHT_COLOUR_START;
    weight /= WEIGHT_COLOUR_END - WEIGHT_COLOUR_START;
    Color::from_rgba(weight, 0., 1. - weight, 1.).unwrap()
}

pub trait DrawPoint {
    fn colour(&self) -> [u8; 3];

    fn paint(&self) -> Paint {
        let [r, g, b] = self.colour();
        let mut paint = Paint::default();
        paint.set_color_rgba8(r, g, b, 255);
        paint
    }

    fn center(&self) -> Vec2d;

    fn path(&self) -> Path {
        let center = self.center();
        PathBuilder::from_circle(center.x, center.y, NODE_RADIUS).unwrap()
    }
}

pub fn draw<N: DrawPoint + HasKey>(graph: &Graph<N, f32>) -> Pixmap where N::Key: PartialOrd {
    let mut pixmap = Pixmap::new(SIZE as u32, SIZE as u32).unwrap();
    pixmap.fill(Color::from_rgba8(BACKGROUND_COLOUR[0], BACKGROUND_COLOUR[1], BACKGROUND_COLOUR[2], 255));
    let transform = Transform::default();
    for from in graph.nodes() {
        for to in graph.nodes() {
            if from.key() <= to.key() {
                continue;   // Only draw each edge once, and ignore equal nodes.
            }
            let weight = graph.get_weight(&from.key(), &to.key());
            if weight < WEIGHT_DISPLAY_THRESHOLD {
                continue;
            }
            let mut path = PathBuilder::new();
            let (from, to) = (from.center(), to.center());
            path.move_to(from.x, from.y);
            path.line_to(to.x, to.y);
            let path = path.finish().unwrap();
            let mut paint = Paint::default();
            paint.set_color(weight_colour(weight));
            let stroke = Stroke { width: EDGE_WIDTH, ..Stroke::default() };
            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
        }
    }
    for node in graph.nodes() {
        pixmap.fill_path(&node.path(), &node.paint(), FillRule::default(), transform, None);
    }
    pixmap
}
