#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    // missing_docs,
    // clippy::missing_docs_in_private_items
)]
use graph::{Graph, HasKey};
use vec2d::Vec2d;

#[cfg(feature = "png")]
use draw::{draw, DrawPoint};

mod graph;
mod vec2d;

#[cfg(feature = "png")]
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
    #[cfg(feature = "png")]
    colour: [u8; 3],
}

impl HasKey for Node {
    type Key = u64;

    fn key(&self) -> Self::Key {
        self.id
    }
}

#[cfg(feature = "png")]
impl DrawPoint for Node {
    fn colour(&self) -> [u8; 3] {
        self.colour
    }

    fn center(&self) -> Vec2d {
        self.pos
    }
}

#[derive(Clone, Debug, Default)]
pub struct System {
    graph: Graph<Node, f32>,
    #[cfg(feature = "lottie")]
    history: lottie_graph::History,
    steps: u64,
}

impl System {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: u64, colour: [u8; 3]) -> u64 {
        let center = Vec2d::new(SIZE / 2., SIZE / 2.);
        let jitter = Vec2d::random_unit() * STARTING_JITTER;
        let pos = center + jitter;
        let velocity = Vec2d::new(0., 0.);
        self.graph.add_node(Node {
            id,
            pos,
            velocity,
            #[cfg(feature = "png")]
            colour,
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
        for (id, accel) in node_accel {
            self.move_node(id, accel);
        }
        self.steps += 1;
        #[cfg(feature = "lottie")]
        self.history.next_step();
        #[cfg(feature = "png")]
        self.render_png_frame();
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
    }

    fn max_distance(&self) -> f32 {
        (self.graph.node_count() as f32).sqrt() * TARGET_DENSITY
    }

    pub fn many_steps(&mut self, count: u64) {
        for _ in 0..count {
            self.step();
        }
    }

    #[cfg(feature = "png")]
    pub fn render_png_frame(&self) {
        draw(&self.graph)
            .save_png(format!("frames/{:0>5}.png", self.steps))
            .unwrap();
    }

    #[cfg(feature = "lottie")]
    pub fn render_lottie(&self) {
        println!("{}", self.history.render().as_json());
    }
}
