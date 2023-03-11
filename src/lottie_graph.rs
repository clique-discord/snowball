use crate::lottie::{Colour, Coords, Ellipse, File, Fill, Keyframe, Layer, Prop, Shape};
use crate::{Vec2d, SIZE};
use hashbrown::HashMap;

const NODE_SIZE: u32 = 20;

#[derive(Clone, Debug)]
struct Frame {
    pos: Coords,
    length: u32,
}

#[derive(Clone, Debug)]
struct Node {
    start: u32,
    colour: Colour,
    frames: Vec<Frame>,
}

impl Node {
    fn push_pos(&mut self, pos: Vec2d) {
        let pos = Coords(pos.x as u32, pos.y as u32);
        if let Some(last) = self.frames.last_mut() {
            if last.pos == pos {
                last.length += 1;
                return;
            }
        }
        self.frames.push(Frame { pos, length: 1 });
    }

    fn render(&self) -> Layer {
        let mut frames = Vec::new();
        let mut time = self.start;
        for frame in &self.frames {
            frames.push(Keyframe {
                time,
                value: frame.pos,
            });
            time += frame.length;
        }
        Layer {
            start: self.start,
            end: time,
            shapes: vec![
                Shape::Ellipse(Ellipse {
                    center: Prop::Animated(frames),
                    size: Prop::Static(Coords(NODE_SIZE, NODE_SIZE)),
                }),
                Shape::Fill(Fill {
                    colour: Prop::Static(self.colour),
                    opacity: Prop::Static(100),
                }),
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct History {
    open: HashMap<u64, Node>,
    closed: Vec<Node>,
    step: u32,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: u64, colour: [u8; 3]) {
        let [r, g, b] = colour;
        let colour = Colour(r as f32 / 255., g as f32 / 255., b as f32 / 255.);
        self.open.insert(
            id,
            Node {
                start: self.step,
                colour,
                frames: Vec::new(),
            },
        );
    }

    pub fn remove_node(&mut self, id: u64) {
        if let Some(node) = self.open.remove(&id) {
            self.closed.push(node);
        }
    }

    pub fn set_position(&mut self, id: u64, pos: Vec2d) {
        self.open.get_mut(&id).unwrap().push_pos(pos);
    }

    pub fn next_step(&mut self) {
        self.step += 1;
    }

    pub fn render(&self) -> File {
        let layers = self
            .closed
            .iter()
            .chain(self.open.values())
            .map(|node| node.render())
            .collect();
        File {
            frame_rate: 60,
            width: SIZE as u32,
            height: SIZE as u32,
            length: self.step,
            layers,
        }
    }
}
