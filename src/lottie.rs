/// A basic implementation of some features of Lottie, a JSON-based vector animation format.
///
/// This does not attempt to cover anywhere near everything Lottie is capable of, only the specific
/// features needed for this project. Additionally, only creating a Lottie file from scratch is
/// supported, there is no deserialization.
use crate::Vec2d;

/// A simple trait for any possible element of a lottie file.
pub trait AsJson {
    /// Returns a string representation of the element as JSON.
    fn as_json(&self) -> String;
}

/// A simple macro for implementing `AsJson` for selected types that can be converted to a string.
macro_rules! to_string_as_json {
    ($($t:ty),*) => {
        $(
            impl AsJson for $t {
                fn as_json(&self) -> String {
                    self.to_string()
                }
            }
        )*
    }
}

to_string_as_json!(u8, u32, u64, f64);

/// Implement `AsJson` for vectors of `AsJson` types by joining the elements with commas.
impl<T: AsJson> AsJson for Vec<T> {
    fn as_json(&self) -> String {
        self
            .iter()
            .map(AsJson::as_json)
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Implement `AsJson` for (x, y) tuples as a two-element array.
impl AsJson for Vec2d {
    fn as_json(&self) -> String {
        format!("[{},{}]", self.x, self.y)
    }
}

/// A keyframe of an animated property.
pub struct Keyframe<T: AsJson> {
    /// The time at which this keyframe occurs.
    pub time: u64,
    /// The value of the property at this keyframe.
    pub value: T,
}

impl<T: AsJson> AsJson for Keyframe<T> {
    fn as_json(&self) -> String {
        let mut value = self.value.as_json();
        // For whatever reason, what would be a non-array type on a static property must be a one
        // element array on an animated property.
        if !value.starts_with('[') {
            value = format!("[{value}]");
        }
        format!(
            r#"{{"t":{t},"i":{{"x":1,"y":1}},"o":{{"x":0,"y":0}},"s":{s}}}"#,
            t = self.time,
            s = value,
        )
    }
}

/// A property of any type that can be either animated or static.
pub enum Prop<T: AsJson> {
    /// A property that is constant over time.
    Static(T),
    /// A property that is animated over time.
    Animated(Vec<Keyframe<T>>),
}

impl<T: AsJson> AsJson for Prop<T> {
    fn as_json(&self) -> String {
        match self {
            Self::Static(v) => format!(r#"{{"a":0,"k":{k}}}"#, k = v.as_json()),
            Self::Animated(v) => format!(r#"{{"a":1,"k":[{k}]}}"#, k = v.as_json()),
        }
    }
}

/// A rectangle shape, defined by its center, width and height.
pub struct Rectangle {
    pub center: Prop<Vec2d>,
    pub size: Prop<Vec2d>,
    pub roundness: Prop<u32>,
}

impl AsJson for Rectangle {
    fn as_json(&self) -> String {
        format!(
            r#"{{"ty":"rc","p":{p},"s":{s},"r":{r}}}"#,
            p = self.center.as_json(),
            s = self.size.as_json(),
            r = self.roundness.as_json(),
        )
    }
}

/// An ellipse shape, defined by its center, width and height.
pub struct Ellipse {
    pub center: Prop<Vec2d>,
    pub size: Prop<Vec2d>,
}

impl AsJson for Ellipse {
    fn as_json(&self) -> String {
        format!(
            r#"{{"ty":"el","p":{p},"s":{s}}}"#,
            p = self.center.as_json(),
            s = self.size.as_json(),
        )
    }
}

/// A path with just two points, and no curves.
///
/// While Lottie supports paths with multiple points and bezier curves, this is all we need for now.
pub struct Segment(pub Vec2d, pub Vec2d);

impl AsJson for Segment {
    fn as_json(&self) -> String {
        // `c: false` means that the path does not form a closed loop.
        // The `i` and `o` fields define the way in which the path curves - the values here mean
        // that the path is straight.
        format!(
            r#"{{"c":false,"v":[{s},{e}],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}}"#,
            s = self.0.as_json(),
            e = self.1.as_json(),
        )
    }
}

/// A line shape, defined by its start and end points.
pub struct Line {
    pub segment: Prop<Segment>
}

impl AsJson for Line {
    fn as_json(&self) -> String {
        format!(r#"{{"ty":"sh","ks":{k}}}"#, k = self.segment.as_json())
    }
}

/// A property type for colours.
///
/// This is a tuple of (R, G, B), where each value is a float between 0 and 1.
pub struct Colour(pub f32, pub f32, pub f32);

/// Implement `AsJson` for colours by returning a JSON array of the RGB values.
impl AsJson for Colour {
    fn as_json(&self) -> String {
        format!("[{},{},{}]", self.0, self.1, self.2)
    }
}

/// A "shape" defining a solid fill for a layer.
pub struct Fill {
    /// The colour of the fill.
    pub colour: Prop<Colour>,
    /// The opacity of the fill, as a percentage (0-100).
    pub opacity: Prop<u8>,
}

impl AsJson for Fill {
    fn as_json(&self) -> String {
        format!(
            r#"{{"ty":"fl","o":{o},"c":{c}}}"#,
            o = self.opacity.as_json(),
            c = self.colour.as_json(),
        )
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

impl AsJson for Stroke {
    fn as_json(&self) -> String {
        format!(
            r#"{{"ty":"st","o":{o},"c":{c},"w":{w}}}"#,
            o = self.opacity.as_json(),
            c = self.colour.as_json(),
            w = self.width.as_json(),
        )
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

impl AsJson for Shape {
    fn as_json(&self) -> String {
        match self {
            Self::Rectangle(r) => r.as_json(),
            Self::Ellipse(e) => e.as_json(),
            Self::Line(l) => l.as_json(),
            Self::Fill(f) => f.as_json(),
            Self::Stroke(s) => s.as_json(),
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
    pub start: u64,
    /// The last frame for which this layer should be visible.
    pub end: u64,
    /// The shapes that make up this layer.
    pub shapes: Vec<Shape>,
}

impl AsJson for Layer {
    fn as_json(&self) -> String {
        format!(
            r#"{{"ip":{i},"op":{e},"st":0,"ks":{{}},"ty":4,"shapes":[{s}]}}"#,
            i = self.start,
            e = self.end,
            s = self.shapes.as_json(),
        )
    }
}

/// A complete Lottie file.
pub struct File {
    pub frame_rate: u32,
    pub width: u32,
    pub height: u32,
    pub length: u64,
    /// The layers, top to bottom.
    pub layers: Vec<Layer>,
}

impl File {
    fn as_json(&self) -> String {
        format!(
            r#"{{"fr":{f},"ip":0,"op":{o},"w":{w},"h":{h},"layers":[{l}]}}"#,
            f = self.frame_rate,
            o = self.length,
            w = self.width,
            h = self.height,
            l = self.layers.as_json(),
        )
    }
}

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

macro_rules! shape {
    ($shape:ident { $( $kind:ident $name:ident $value:tt )* }) => {
        $crate::lottie::Shape::$shape($crate::lottie::$shape {
            $( $name: prop!($kind $value), )*
        })
    };
}

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

#[cfg(test)]
mod tests {
    use super::{File, Segment, Vec2d, Colour};

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
                        static segment { Segment(Vec2d::new(128., 256.), Vec2d::new(384., 256.)) }
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
                            0 => Segment(Vec2d::new(0., 0.), Vec2d::new(512., 512.)),
                            30 => Segment(Vec2d::new(512., 0.), Vec2d::new(0., 512.)),
                            60 => Segment(Vec2d::new(512., 512.), Vec2d::new(0., 0.)),
                            90 => Segment(Vec2d::new(0., 512.), Vec2d::new(512., 0.)),
                            120 => Segment(Vec2d::new(0., 0.), Vec2d::new(512., 512.)),
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
                            30 => Vec2d::new(64., 64.),
                            60 => Vec2d::new(448., 64.),
                        }
                        static size { Vec2d::new(64., 64.) }
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
                            90 => Vec2d::new(448., 448.),
                            120 => Vec2d::new(64., 448.),
                        }
                        static size { Vec2d::new(64., 64.) }
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
                        static center { Vec2d::new(256., 256.) }
                        keyframes size {
                            0 => Vec2d::new(0., 0.),
                            100 => Vec2d::new(362., 362.),
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
                        static center { Vec2d::new(256., 256.) }
                        static size { Vec2d::new(512., 512.) }
                        static roundness { 0 }
                    }
                    Fill {
                        static colour { Colour(1., 0., 0.) }
                        static opacity { 100 }
                    }
                },
            ],
        };
        println!("{}", file.as_json());
        assert_eq!(
            file.as_json(),
            r#"{"fr":60,"ip":0,"op":120,"w":512,"h":512,"layers":[{"ip":0,"op":60,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"sh","ks":{"a":0,"k":{"c":false,"v":[[128,256],[384,256]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[0,0,0]},"w":{"a":0,"k":1}}]},{"ip":0,"op":120,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"sh","ks":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[0,0],[512,512]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":30,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[512,0],[0,512]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":60,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[512,512],[0,0]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":90,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[0,512],[512,0]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]},{"t":120,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[{"c":false,"v":[[0,0],[512,512]],"i":[[0,0],[0,0]],"o":[[0,0],[0,0]]}]}]}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[1,1,0]},"w":{"a":0,"k":16}}]},{"ip":30,"op":60,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"el","p":{"a":1,"k":[{"t":30,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[64,64]},{"t":60,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[448,64]}]},"s":{"a":0,"k":[64,64]}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[0,0,1]},"w":{"a":0,"k":8}},{"ty":"fl","o":{"a":0,"k":50},"c":{"a":0,"k":[0,1,0]}}]},{"ip":90,"op":120,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"el","p":{"a":1,"k":[{"t":90,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[448,448]},{"t":120,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[64,448]}]},"s":{"a":0,"k":[64,64]}},{"ty":"st","o":{"a":0,"k":100},"c":{"a":0,"k":[0,1,0]},"w":{"a":0,"k":8}},{"ty":"fl","o":{"a":0,"k":50},"c":{"a":0,"k":[0,0,1]}}]},{"ip":0,"op":100,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"el","p":{"a":0,"k":[256,256]},"s":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[0,0]},{"t":100,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[362,362]}]}},{"ty":"fl","o":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[0]},{"t":100,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[100]}]},"c":{"a":1,"k":[{"t":0,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[0,0,0]},{"t":100,"i":{"x":1,"y":1},"o":{"x":0,"y":0},"s":[1,1,1]}]}}]},{"ip":0,"op":120,"st":0,"ks":{},"ty":4,"shapes":[{"ty":"rc","p":{"a":0,"k":[256,256]},"s":{"a":0,"k":[512,512]},"r":{"a":0,"k":0}},{"ty":"fl","o":{"a":0,"k":100},"c":{"a":0,"k":[1,0,0]}}]}]}"#
        );
    }
}
