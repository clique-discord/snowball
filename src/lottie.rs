//! A basic implementation of some features of Lottie, a JSON-based vector animation format.
//!
//! This does not attempt to cover anywhere near everything Lottie is capable of, only the specific
//! features needed for this project. Additionally, only creating a Lottie file from scratch is
//! supported, there is no deserialization.
//!
//! # Numbers
//!
//! Lottie seems to accept fractional numbers almost everywhere, so we could just use `f32` for
//! everything. However, for our purposes integer values suffice for coordinates and dimensions,
//! and we need to truncate at some precision anyway to keep files small. Therefore we use `u32` for
//! all coordinates and dimensions. We also use `u32` for frame numbers (theoretically allowing an
//! almost 3 year animation at 60fps).
//!
//! Colours in Lottie are represented by values in the 0-1 range, so we use `f32` for them. Opacity
//! on the other hand is represented by values in the 0-100 range, which we use a `u8` for.
use std::fmt::Write;

/// A simple trait for any possible element of a lottie file.
pub trait WriteJson {
    /// Write a JSON representation of the element to a buffer.
    fn write_json(&self, s: &mut String);
}

/// A simple macro for implementing `WriteJson` for selected types that can be converted to a string.
macro_rules! to_string_write_json {
    ($($t:ty),*) => {
        $(
            impl WriteJson for $t {
                fn write_json(&self, s: &mut String) {
                    write!(s, "{}", self).unwrap();
                }
            }
        )*
    }
}

to_string_write_json!(u8, u32);

/// Implement `WriteJson` for vectors of `WriteJson` types by joining the elements with commas.
impl<T: WriteJson> WriteJson for Vec<T> {
    fn write_json(&self, s: &mut String) {
        let mut first = true;
        for v in self {
            if first {
                first = false;
            } else {
                s.push(',');
            }
            v.write_json(s);
        }
    }
}

/// Integer coordinates.
///
/// Lottie seems to accept fractional numbers almost everywhere, but we want to truncate anyway to
/// reduce file size, and integers provide performance benefits too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coords(pub u32, pub u32);

impl WriteJson for Coords {
    fn write_json(&self, s: &mut String) {
        write!(s, "[{},{}]", self.0, self.1).unwrap();
    }
}

/// A keyframe of an animated property.
pub struct Keyframe<T: WriteJson> {
    /// The time at which this keyframe occurs.
    pub time: u32,
    /// The value of the property at this keyframe.
    pub value: T,
}

impl<T: WriteJson> WriteJson for Keyframe<T> {
    fn write_json(&self, s: &mut String) {
        let mut value_buffer = String::new();
        self.value.write_json(&mut value_buffer);
        // For whatever reason, what would be a non-array type on a static property must be a one
        // element array on an animated property. Therefore we must check if the value is already an
        // array, and if not, wrap it in one.
        if !value_buffer.starts_with('[') {
            value_buffer.insert(0, '[');
            value_buffer.push(']');
        }
        write!(
            s,
            r#"{{"t":{t},"i":{{"x":1,"y":1}},"o":{{"x":0,"y":0}},"s":{value}}}"#,
            t = self.time,
            value = value_buffer,
        )
        .unwrap();
    }
}

/// A property of any type that can be either animated or static.
pub enum Prop<T: WriteJson> {
    /// A property that is constant over time.
    Static(T),
    /// A property that is animated over time.
    Animated(Vec<Keyframe<T>>),
}

impl<T: WriteJson> WriteJson for Prop<T> {
    fn write_json(&self, s: &mut String) {
        match self {
            Self::Static(v) => {
                s.push_str(r#"{"a":0,"k":"#);
                v.write_json(s);
                s.push('}');
            }
            Self::Animated(v) => {
                s.push_str(r#"{"a":1,"k":["#);
                v.write_json(s);
                s.push_str("]}");
            }
        }
    }
}

/// A rectangle shape, defined by its center, width and height.
pub struct Rectangle {
    pub center: Prop<Coords>,
    pub size: Prop<Coords>,
    pub roundness: Prop<u32>,
}

impl WriteJson for Rectangle {
    fn write_json(&self, s: &mut String) {
        s.push_str(r#"{"ty":"rc","p":"#);
        self.center.write_json(s);
        s.push_str(r#","s":"#);
        self.size.write_json(s);
        s.push_str(r#","r":"#);
        self.roundness.write_json(s);
        s.push('}');
    }
}

/// An ellipse shape, defined by its center, width and height.
pub struct Ellipse {
    pub center: Prop<Coords>,
    pub size: Prop<Coords>,
}

impl WriteJson for Ellipse {
    fn write_json(&self, s: &mut String) {
        s.push_str(r#"{"ty":"el","p":"#);
        self.center.write_json(s);
        s.push_str(r#","s":"#);
        self.size.write_json(s);
        s.push('}');
    }
}

/// A path with just two points, and no curves.
///
/// While Lottie supports paths with multiple points and bezier curves, this is all we need for now.
pub struct Segment(pub Coords, pub Coords);

impl WriteJson for Segment {
    fn write_json(&self, s: &mut String) {
        // `c: false` means that the path does not form a closed loop.
        // The `i` and `o` fields define the way in which the path curves - the values here mean
        // that the path is straight.
        s.push_str(r#"{"c":false,"i":[[0,0],[0,0]],"o":[[0,0],[0,0]],"v":["#);
        self.0.write_json(s);
        s.push(',');
        self.1.write_json(s);
        s.push_str("]}");
    }
}

/// A line shape, defined by its start and end points.
pub struct Line {
    pub segment: Prop<Segment>,
}

impl WriteJson for Line {
    fn write_json(&self, s: &mut String) {
        s.push_str(r#"{"ty":"fl","ks":"#);
        self.segment.write_json(s);
        s.push('}');
    }
}

/// A property type for colours.
///
/// This is a tuple of (R, G, B), where each value is a float between 0 and 1.
#[derive(Clone, Copy, Debug)]
pub struct Colour(pub f32, pub f32, pub f32);

/// Implement `AsJson` for colours by returning a JSON array of the RGB values.
impl WriteJson for Colour {
    fn write_json(&self, s: &mut String) {
        write!(s, "[{:.3},{:.3},{:.3}]", self.0, self.1, self.2).unwrap();
    }
}

/// A "shape" defining a solid fill for a layer.
pub struct Fill {
    /// The colour of the fill.
    pub colour: Prop<Colour>,
    /// The opacity of the fill, as a percentage (0-100).
    pub opacity: Prop<u8>,
}

impl WriteJson for Fill {
    fn write_json(&self, s: &mut String) {
        s.push_str(r#"{"ty":"fl","o":"#);
        self.opacity.write_json(s);
        s.push_str(r#","c":"#);
        self.colour.write_json(s);
        s.push('}');
    }
}

/// A "shape" defining a solid stroke for a layer.
pub struct Stroke {
    /// The colour of the stroke.
    pub colour: Prop<Colour>,
    /// The opacity of the stroke, as a percentage (0-100).
    pub opacity: Prop<u8>,
    /// The width of the stroke, in pixels.
    pub width: Prop<u32>,
}

impl WriteJson for Stroke {
    fn write_json(&self, s: &mut String) {
        s.push_str(r#"{"ty":"st","o":"#);
        self.opacity.write_json(s);
        s.push_str(r#","c":"#);
        self.colour.write_json(s);
        s.push_str(r#","w":"#);
        self.width.write_json(s);
        s.push('}');
    }
}

/// A "shape" used to define part of a layer.
///
/// In Lottie, "shape" refers to any vector related data. This includes actual shapes, as well as
/// styles applied to those shapes. Other kinds of shape also exist but we don't need them.
pub enum Shape {
    Rectangle(Rectangle),
    Ellipse(Ellipse),
    Line(Line),
    Fill(Fill),
    Stroke(Stroke),
}

impl WriteJson for Shape {
    fn write_json(&self, s: &mut String) {
        match self {
            Self::Rectangle(r) => r.write_json(s),
            Self::Ellipse(e) => e.write_json(s),
            Self::Line(l) => l.write_json(s),
            Self::Fill(f) => f.write_json(s),
            Self::Stroke(st) => st.write_json(s),
        }
    }
}

/// A layer in a Lottie file.
///
/// For our purposes, a layer will typically include two or three "shapes": an actual shape,
/// followed by a fill style and/or a stroke style.
pub struct Layer {
    /// The first frame for which this layer should be visible.
    ///
    /// Note that this corresponds to the `ip` ("in point") field in Lottie, not
    /// `st` ("start time"). We always set the `st` field to `0`.
    pub start: u32,
    /// The last frame for which this layer should be visible.
    pub end: u32,
    /// The shapes that make up this layer.
    pub shapes: Vec<Shape>,
}

impl WriteJson for Layer {
    fn write_json(&self, s: &mut String) {
        s.push_str(r#"{"ip":"#);
        write!(s, "{}", self.start).unwrap();
        s.push_str(r#","op":"#);
        write!(s, "{}", self.end).unwrap();
        s.push_str(r#","st":0,"ks":{},"ty":4,"shapes":["#);
        self.shapes.write_json(s);
        s.push_str("]}");
    }
}

/// A complete Lottie file.
pub struct File {
    pub frame_rate: u32,
    pub width: u32,
    pub height: u32,
    pub length: u32,
    /// The layers, top to bottom.
    pub layers: Vec<Layer>,
}

impl File {
    pub fn as_json(&self) -> String {
        let mut s = String::new();
        s.push_str(r#"{"fr":"#);
        write!(s, "{}", self.frame_rate).unwrap();
        s.push_str(r#","ip":0,"op":"#);
        write!(s, "{}", self.length).unwrap();
        s.push_str(r#","w":"#);
        write!(s, "{}", self.width).unwrap();
        s.push_str(r#","h":"#);
        write!(s, "{}", self.height).unwrap();
        s.push_str(r#","layers":["#);
        self.layers.write_json(&mut s);
        s.push_str("]}");
        s
    }
}

#[macro_export]
macro_rules! prop {
    (static { $value:expr }) => {
        $crate::lottie::Prop::Static($value)
    };

    (keyframes { $( $time:expr => $value:expr, )* }) => {
        $crate::lottie::Prop::Animated(vec![ $(
            $crate::lottie::Keyframe {
                time: $time,
                value: $value,
            },
        )* ])
    };
}

#[macro_export]
macro_rules! shape {
    ($shape:ident { $( $kind:ident $name:ident $value:tt )* }) => {
        $crate::lottie::Shape::$shape($crate::lottie::$shape {
            $( $name: prop!($kind $value), )*
        })
    };
}

#[macro_export]
macro_rules! layer {
    (($start:expr; $end:expr) $( $name:ident $props:tt )*) => {
        $crate::lottie::Layer {
            start: $start,
            end: $end,
            shapes: vec![
                $( shape!($name $props), )*
            ],
        }
    };
}

pub use {layer, prop, shape};

#[cfg(test)]
mod tests {
    use super::{Colour, Coords, File, Segment};

    #[test]
    fn entire_file() {
        let file = File {
            frame_rate: 60,
            width: 512,
            height: 512,
            length: 120,
            layers: vec![
                layer! {
                    (0; 60)
                    Line {
                        static segment { Segment(Coords(128, 256), Coords(384, 256)) }
                    }
                    Stroke {
                        static colour { Colour(0., 0., 0.) }
                        static opacity { 100 }
                        static width { 1 }
                    }
                },
                layer! {
                    (0; 120)
                    Line {
                        keyframes segment {
                            0 => Segment(Coords(0, 0), Coords(512, 512)),
                            30 => Segment(Coords(512, 0), Coords(0, 512)),
                            60 => Segment(Coords(512, 512), Coords(0, 0)),
                            90 => Segment(Coords(0, 512), Coords(512, 0)),
                            120 => Segment(Coords(0, 0), Coords(512, 512)),
                        }
                    }
                    Stroke {
                        static colour { Colour(1., 1., 0.) }
                        static opacity { 100 }
                        static width { 16 }
                    }
                },
                layer! {
                    (30; 60)
                    Ellipse {
                        keyframes center {
                            30 => Coords(64, 64),
                            60 => Coords(448, 64),
                        }
                        static size { Coords(64, 64) }
                    }
                    Stroke {
                        static colour { Colour(0., 0., 1.) }
                        static opacity { 100 }
                        static width { 8 }
                    }
                    Fill {
                        static colour { Colour(0., 1., 0.) }
                        static opacity { 50 }
                    }
                },
                layer! {
                    (90; 120)
                    Ellipse {
                        keyframes center {
                            90 => Coords(448, 448),
                            120 => Coords(64, 448),
                        }
                        static size { Coords(64, 64) }
                    }
                    Stroke {
                        static colour { Colour(0., 1., 0.) }
                        static opacity { 100 }
                        static width { 8 }
                    }
                    Fill {
                        static colour { Colour(0., 0., 1.) }
                        static opacity { 50 }
                    }
                },
                layer! {
                    (0; 100)
                    Ellipse {
                        static center { Coords(256, 256) }
                        keyframes size {
                            0 => Coords(0, 0),
                            100 => Coords(362, 362),
                        }
                    }
                    Fill {
                        keyframes colour {
                            0 => Colour(0., 0., 0.),
                            100 => Colour(1., 1., 1.),
                        }
                        keyframes opacity {
                            0 => 0,
                            100 => 100,
                        }
                    }
                },
                layer! {
                    (0; 120)
                    Rectangle {
                        static center { Coords(256, 256) }
                        static size { Coords(512, 512) }
                        static roundness { 0 }
                    }
                    Fill {
                        static colour { Colour(1., 0., 0.) }
                        static opacity { 100 }
                    }
                },
            ],
        };
        assert_eq!(
            file.as_json(),
            r#"{"fr":60,"ip":0,"op":120,"w":512,"h":512,"layers":[{"ip":0,"op":60,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"sh","ks":{"a":0,"k":{"c":false,"v":[[128,256],[384,256]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[0,0,0]},"w":{"a":0,"k":1}}]},{"ip":0,"op":120,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"sh","ks":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[0,0],[512,512]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":30,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[512,0],[0,512]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":60,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[512,512],[0,0]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":90,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[0,512],[512,0]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":120,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[0,0],[512,512]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]}]}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[1,1,0]},"w":{"a":0,"k":16}}]},{"ip":30,"op":60,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"el","p":{"a":1,"k":[{"t":30,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[64,64]},{"t":60,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[448,64]}]},"s":{"a":0,"k":[64,64]}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[0,0,1]},"w":{"a":0,"k":8}},{"ty":"fl","o":{"a":0,"k":50},"c":{"a":0,"k":[0,1,0]}}]},{"ip":90,"op":120,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"el","p":{"a":1,"k":[{"t":90,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[448,448]},{"t":120,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[64,448]}]},"s":{"a":0,"k":[64,64]}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[0,1,0]},"w":{"a":0,"k":8}},{"ty":"fl","o":{"a":0,"k":50},"c":{"a":0,"k":[0,0,1]}}]},{"ip":0,"op":100,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"el","p":{"a":0,"k":[256,256]},"s":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[0,0]},{"t":100,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[362,362]}]}},{"ty":"fl","o":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[0]},{"t":100,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[100]}]},"c":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[0,0,0]},{"t":100,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[1,1,1]}]}}]},{"ip":0,"op":120,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"rc","p":{"a":0,"k":[256,256]},"s":{"a":0,"k":[512,512]},"r":{"a":0,"k":0}},{"ty":"fl","o":{"a":0,"k":100},"c":{"a":0,"k":[1,0,0]}}]}]}"#
        );
    }
}
