#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    // missing_docs,
    // clippy::missing_docs_in_private_items
)]
use graph::{Graph, HasKey};
use std::fs::File;
use std::io::BufWriter;
use vec2d::Vec2d;

#[cfg(feature = "raster")]
use draw::{Drawing, Order};

mod graph;
#[cfg(feature = "masquerade")]
mod masquerade;
mod vec2d;

#[cfg(feature = "raster")]
mod draw;

#[cfg(feature = "lottie")]
mod lottie;
#[cfg(feature = "lottie")]
mod lottie_graph;

const SPRING_CONSTANT: f32 = 0.01;
const TARGET_DENSITY: f32 = 150.;
const MIN_SPRING_LENGTH: f32 = 10.;
const DAMPING: f32 = 0.9;
const SIZE: f32 = 1000.;
const STARTING_JITTER: f32 = 5.;

#[derive(Clone, Debug)]
struct Node {
    id: u64,
    pos: Vec2d,
    velocity: Vec2d,
    #[cfg(feature = "raster")]
    order: Order,
    #[cfg(feature = "masquerade")]
    palette_index: u8,
}

impl HasKey for Node {
    type Key = u64;

    fn key(&self) -> Self::Key {
        self.id
    }
}

pub struct System {
    graph: Graph<Node, f32>,
    #[cfg(feature = "lottie")]
    history: lottie_graph::History,
    #[cfg(feature = "raster")]
    drawing: Drawing,
    #[cfg(feature = "gif")]
    gif: gif::Encoder<BufWriter<File>>,
    steps: u64,
    #[cfg(feature = "masquerade")]
    im: masquerade::Image,
}

impl System {
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            #[cfg(feature = "lottie")]
            history: lottie_graph::History::new(),
            #[cfg(feature = "raster")]
            drawing: Drawing::new(),
            #[cfg(feature = "gif")]
            gif: gif::Encoder::new(
                BufWriter::new(File::create("out.gif").unwrap()),
                SIZE as u16,
                SIZE as u16,
                &[],
            )
            .unwrap(),
            steps: 0,
            #[cfg(feature = "masquerade")]
            im: masquerade::Image::new(),
        }
    }

    pub fn add_node(&mut self, id: u64, colour: [u8; 3]) -> u64 {
        let center = Vec2d::new(SIZE / 2., SIZE / 2.);
        let jitter = Vec2d::random_unit() * STARTING_JITTER;
        let pos = center + jitter;
        let velocity = Vec2d::new(0., 0.);
        #[cfg(feature = "raster")]
        let order = self.drawing.add_node(colour);
        #[cfg(feature = "masquerade")]
        let palette_index = self.im.add_node(colour);
        self.graph.add_node(Node {
            id,
            pos,
            velocity,
            #[cfg(feature = "raster")]
            order,
            #[cfg(feature = "masquerade")]
            palette_index,
        });
        #[cfg(feature = "lottie")]
        self.history.add_node(id, colour);
        id
    }

    pub fn set_weight(&mut self, from: u64, to: u64, weight: f32) {
        self.graph.set_weight(from, to, weight);
    }

    pub fn step(&mut self) {
        // First calculate the acceleration for each node, then apply it.
        // This is necessary because the acceleration depends on the positions of all nodes.
        let mut node_accel = Vec::with_capacity(self.graph.node_count());
        for node in self.graph.nodes() {
            node_accel.push((node.id, self.node_acceleration(node)));
        }
        #[cfg(feature = "masquerade")]
        self.im.new_frame();
        for (id, accel) in node_accel {
            self.move_node(id, accel);
        }
        self.steps += 1;
        #[cfg(feature = "lottie")]
        self.history.next_step();
        #[cfg(feature = "raster")]
        self.render_raster_frame();
    }

    fn node_acceleration(&self, node: &Node) -> Vec2d {
        let mut accel = Vec2d::new(0., 0.);
        for (sibling, weight) in self.graph.edges(node.id) {
            let spring_length = (self.max_distance() - weight).max(MIN_SPRING_LENGTH);
            let force = SPRING_CONSTANT * (node.pos.distance(sibling.pos) - spring_length);
            let direction = (sibling.pos - node.pos).as_unit();
            accel += direction * force;
        }
        accel
    }

    fn move_node(&mut self, id: u64, accel: Vec2d) {
        let node = self.graph.get_node_mut(&id).unwrap();
        node.velocity += accel;
        node.velocity *= DAMPING;
        node.pos += node.velocity;
        #[cfg(feature = "lottie")]
        self.history.set_position(id, node.pos);
        #[cfg(feature = "raster")]
        self.drawing.place_node(node.order, node.pos);
        #[cfg(feature = "masquerade")]
        self.im.place_node(node.palette_index, node.pos);
    }

    fn max_distance(&self) -> f32 {
        (self.graph.node_count() as f32).sqrt() * TARGET_DENSITY
    }

    pub fn many_steps(&mut self, count: u64) {
        for _ in 0..count {
            self.step();
        }
    }

    #[cfg(feature = "raster")]
    fn render_raster_frame(&mut self) {
        self.drawing.render_frame();
        #[cfg(feature = "png")]
        self.render_png_frame();
        #[cfg(feature = "gif")]
        self.render_gif_frame();
    }

    #[cfg(feature = "png")]
    fn render_png_frame(&mut self) {
        let mut file = File::create(format!("frames/frame{:04}.png", self.steps)).unwrap();
        let mut buf_writer = BufWriter::new(&mut file);
        self.drawing.frame_as_png(&mut buf_writer);
    }

    #[cfg(feature = "gif")]
    fn render_gif_frame(&mut self) {
        let mut frame = self.drawing.frame_as_gif();
        frame.delay = 2;
        self.gif.write_frame(&frame).unwrap();
    }

    #[cfg(feature = "lottie")]
    pub fn render_lottie(&self) {
        println!("{}", self.history.render().as_json());
    }

    #[cfg(feature = "masquerade")]
    pub fn render_masquerade(&self) {
        let file = File::create("test.gif").unwrap();
        let mut buf_writer = BufWriter::new(file);
        self.im.render(&mut buf_writer);
    }
}
