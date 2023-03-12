use crate::Vec2d;
use rayon::prelude::*;
use std::io::Write;

const NODE_RADIUS: usize = 10;
const IMAGE_SIZE: usize = 1000;
const BACKGROUND_COLOUR: [u8; 3] = [238, 232, 213];

const NODE_MASK: [[bool; 2 * NODE_RADIUS]; 2 * NODE_RADIUS] = {
    let mut mask = [[false; 2 * NODE_RADIUS]; 2 * NODE_RADIUS];
    let mut y = 0;
    while y < 2 * NODE_RADIUS {
        let mut x = 0;
        while x < 2 * NODE_RADIUS {
            let dx = x.abs_diff(NODE_RADIUS);
            let dy = y.abs_diff(NODE_RADIUS);
            mask[y][x] = dx * dx + dy * dy <= NODE_RADIUS * NODE_RADIUS;
            x += 1;
        }
        y += 1;
    }
    mask
};

struct Node {
    palette_index: u8,
    pos: Vec2d,
}

impl Node {
    fn draw(&self, image: &mut [u8; IMAGE_SIZE * IMAGE_SIZE]) {
        let start_x = (self.pos.x as usize).saturating_sub(NODE_RADIUS);
        let start_y = (self.pos.y as usize).saturating_sub(NODE_RADIUS);
        for x in 0..2 * NODE_RADIUS {
            for y in 0..2 * NODE_RADIUS {
                if NODE_MASK[y][x] {
                    let image_x = start_x + x;
                    let image_y = start_y + y;
                    let index = image_x + image_y * IMAGE_SIZE;
                    image[index] = self.palette_index;
                }
            }
        }
    }
}

pub struct Image {
    frames: Vec<Vec<Node>>,
    palette: Vec<[u8; 3]>,
}

impl Image {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            palette: vec![BACKGROUND_COLOUR],
        }
    }

    pub fn add_node(&mut self, colour: [u8; 3]) -> u8 {
        if let Some(index) = self.palette.iter().position(|c| *c == colour) {
            index as u8
        } else {
            let index = self.palette.len();
            self.palette.push(colour);
            index as u8
        }
    }

    pub fn place_node(&mut self, palette_index: u8, pos: Vec2d) {
        self.frames
            .last_mut()
            .unwrap()
            .push(Node { palette_index, pos });
    }

    pub fn new_frame(&mut self) {
        self.frames.push(Vec::new());
    }

    pub fn render(&self, w: impl Write) {
        let palette = self.palette.iter().flatten().copied().collect::<Vec<_>>();
        let mut gif = gif::Encoder::new(w, IMAGE_SIZE as u16, IMAGE_SIZE as u16, &palette).unwrap();
        let base_image = [0; IMAGE_SIZE * IMAGE_SIZE];
        let mut frames = Vec::with_capacity(self.frames.len());
        self.frames
            .par_iter()
            .map(|frame| {
                let mut image = base_image.clone();
                for node in frame {
                    node.draw(&mut image);
                }
                let mut frame = gif::Frame::from_indexed_pixels(
                    IMAGE_SIZE as u16,
                    IMAGE_SIZE as u16,
                    &image,
                    None,
                );
                frame.delay = 2;
                frame.make_lzw_pre_encoded();
                frame
            })
            .collect_into_vec(&mut frames);
        frames
            .into_iter()
            .for_each(|frame| gif.write_lzw_pre_encoded_frame(&frame).unwrap());
    }
}
