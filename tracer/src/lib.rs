#![feature(field_init_shorthand, const_fn)]

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

#[derive(Debug, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    const fn new(r: u8,
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

#[derive(Debug)]
struct Ray {
    position: WorldPosition,
    direction: Point3D<f32>,
    strength: f32,
}

fn length<U>(p: TypedPoint3D<f32, U>) -> f32 {
    (p.x * p.x + p.y * p.y + p.z * p.z).sqrt()
}

fn scale<U>(v: TypedPoint3D<f32, U>, factor: f32) -> TypedPoint3D<f32, U> {
    TypedPoint3D::new(v.x * factor, v.y * factor, v.z * factor)
}

fn to_world_units<U>(v: TypedPoint3D<f32, U>) -> WorldPosition {
    TypedPoint3D::new(v.x, v.y, v.z)
}

fn reflect<U>(dir: TypedPoint3D<f32, U>,
              normal: TypedPoint3D<f32, U>)
              -> TypedPoint3D<f32, U> {
    dir - scale(normal, normal.dot(dir) * 2.0)
}

impl Ray {
    fn new(position: WorldPosition,
           direction: Point3D<f32>,
           strength: f32) -> Self {
        Ray { position, direction, strength }
    }

    // Tries to calculate the ray intersection between this ray and a given
    // object, and returns the reflected ray, if any.
    fn intersects_with(&self, object: &Object) -> Option<Ray> {
        match object.kind {
            ObjectKind::Sphere  { radius } => {
                // https://en.wikipedia.org/wiki/Line%E2%80%93sphere_intersection
                //
                // this is |l \dot (o - c)|
                let b =
                    self.direction.dot((self.position - object.position).to_untyped());
                if !b.is_finite() {
                    return None;
                }

                // This is the value under the square root.
                let v = b * b -
                    length(self.position - object.position).powf(2.0)
                    + radius.powf(2.0);

                if v < 0.0 {
                    return None;
                }

                // TODO(emilio): This should be in the material of the object.
                const OBJECT_REFLECTIVENESS: f32 = 0.5;

                // Tangent intersection -> same ray
                if v == 0.0 {
                    let distance = -b;
                    let position = self.position + to_world_units(scale(self.direction, distance));
                    return Some(Ray::new(position,
                                         self.direction,
                                         self.strength * OBJECT_REFLECTIVENESS));
                }

                let v = v.sqrt();
                let distance = (-b + v).min(-b - v);
                if !distance.is_finite() {
                    return None;
                }

                let position = self.position + to_world_units(scale(self.direction, distance));
                let normal = position - object.position;
                let new_direction = reflect(self.direction, normal.to_untyped());

                Some(Ray::new(position,
                              new_direction,
                              self.strength * OBJECT_REFLECTIVENESS))
            }
        }
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

    pub fn size(&self) -> Size2D<usize> {
        self.size
    }

    fn get_color_at(&self,
                    framebuffer: &[u8],
                    x: usize,
                    y: usize) -> Color {
        let slice = &framebuffer[x * 4 + y * self.size.width * 4..];

        Color::new(slice[0], slice[1], slice[2], slice[3])
    }


    fn set_color_at(&self,
                    framebuffer: &mut [u8],
                    x: usize,
                    y: usize,
                    color: Color) {
        let slice = &mut framebuffer[x * 4 + y * self.size.width * 4..];
        slice[0] = color.r;
        slice[1] = color.g;
        slice[2] = color.b;
        slice[3] = color.a;
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

        self.set_color_at(&mut framebuffer, self.size.width - 1, self.size.height - 1, Color::new(0, 0, 0, 255));
        assert_eq!(framebuffer[framebuffer.len() - 1], 255);


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
        const MAX_REFLECTIONS: usize = 10;

        let mut current_ray = Ray::new(
            WorldPosition::new(x as f32, y as f32, -1000.0),
            Point3D::new(0.0, 0.0, 1.0),
            1.0);

        let mut current_intersection: Option<(Ray, &Object)> = None;
        let mut current_reflection = 0;

        while current_reflection <= MAX_REFLECTIONS && current_ray.strength != 0.0 {
            for object in &self.objects {
                let intersection = match current_ray.intersects_with(object) {
                    None => continue,
                    Some(i) => i,
                };

                let should_override_current = match current_intersection {
                    Some((ref current, _)) => intersection.position.z < current.position.z,
                    None => true,
                };

                if should_override_current {
                    current_intersection = Some((intersection, object));
                }
            }

            match current_intersection.take() {
                Some((current, _object)) => {
                    // TODO(emilio): get this from the object.
                    const OBJECT_COLOR: Color = Color::new(255, 255, 0, 255);
                    println!("Current: {:?}", current);
                    let color = self.get_color_at(framebuffer, x, y);
                    self.set_color_at(framebuffer, x, y,
                                 Color::new(
                                     (color.r as f32 + OBJECT_COLOR.r as f32 * current.strength) as u8,
                                     (color.g as f32 + OBJECT_COLOR.g as f32 * current.strength) as u8,
                                     (color.b as f32 + OBJECT_COLOR.b as f32 * current.strength) as u8,
                                     (color.a as f32 + OBJECT_COLOR.a as f32 * current.strength) as u8)
                                 );

                    current_ray = current;
                }
                None => break,
            }

            current_reflection += 1;
        }
    }
}
