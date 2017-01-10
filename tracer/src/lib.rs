#![feature(field_init_shorthand)]

extern crate euclid;

use euclid::{Size2D, Point3D, TypedPoint3D};

pub enum WorldUnits {}
pub type WorldPosition = TypedPoint3D<f32, WorldUnits>;

pub enum ObjectKind {
    Sphere { radius: f32, }
}

pub struct Object {
    kind: ObjectKind,
    position: WorldPosition,
}

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn new(r: u8,
           g: u8,
           b: u8,
           a: u8) -> Self {
        Color { r, g, b, a }
    }
}

pub struct Material {
    diffuse: Color
}

impl Object {
    // TODO(emilo): Materials
    pub fn new(kind: ObjectKind,
               position: WorldPosition)
               -> Self {
        Object { kind, position, }
    }
}

struct Ray {
    position: WorldPosition,
    direction: Point3D<f32>,
    strength: f32,
}

struct Intersection {
    position: WorldPosition,
    normal: Point3D<f32>,
}

impl Ray {
    fn new(position: WorldPosition,
               direction: Point3D<f32>,
               strength: f32) -> Self {
        Ray { position, direction, strength }
    }

    fn intersects_with(&self, object: &Object) -> Option<Intersection> {
        // TODO(emilio)
        None
    }
}


pub struct Scene {
    size: Size2D<usize>,
    camera_position: WorldPosition,
    light_position: WorldPosition,
    objects: Vec<Object>,
}

impl Scene {
    pub fn new(size: Size2D<usize>,
               camera_position: WorldPosition,
               light_position: WorldPosition)
               -> Self {
        Scene { size, camera_position, light_position, objects: vec![], }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object)
    }

    pub fn draw(&self) -> Vec<u8> {
        // TODO(emilio): Maybe organizing this with transforms instead of
        // position in world space + radius, then allowing customizing a bunch
        // of stuff.
        let mut framebuffer = vec![0u8; self.size.width * self.size.height * 4];
        let mut z_buffer = vec![0f32; self.size.width * self.size.height];

        for x in 0..self.size.width {
            for y in 0..self.size.height {
                self.cast_ray_from(x, y, &mut framebuffer, &mut z_buffer);
            }
        }

        framebuffer
    }

    fn cast_ray_from(&self,
                     x: usize,
                     y: usize,
                     framebuffer: &mut [u8],
                     z_buffer: &mut [f32]) {
        // TODO.
    }
}
