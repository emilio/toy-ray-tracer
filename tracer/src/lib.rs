#![feature(field_init_shorthand, const_fn)]

extern crate euclid;

use euclid::{Size2D, Point3D, TypedPoint3D};
use std::ops;

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
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    const fn new(r: f32,
                 g: f32,
                 b: f32,
                 a: f32) -> Self {
        Color { r, g, b, a }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self {
        Color::new(self.r * rhs,
                   self.g * rhs,
                   self.b * rhs,
                   self.a)
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self {
        Color::new(self.r + rhs.r,
                   self.g + rhs.g,
                   self.b + rhs.b,
                   self.a + rhs.a)
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
                    length(self.position - object.position).powi(2)
                    + radius.powi(2);

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
                let solve = |dist: f32| {
                    if !dist.is_finite() {
                        None
                    } else {
                        Some(to_world_units(scale(self.direction, dist)))
                    }
                };

                // The two different solutions are -b + v and -b -v, chose the
                // closer one to the origin of the ray.
                let solution = if (-b + v).abs() < (-b - v).abs() {
                    - b + v
                } else {
                    - b - v
                };

                let position = match solve(solution) {
                    Some(s) => self.position + s,
                    None => return None,
                };

                let normal = normalize(position - object.position);
                let new_direction = reflect(self.direction, normal.to_untyped());

                Some(Ray::new(position,
                              normalize(new_direction),
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

fn normalize<U>(v: TypedPoint3D<f32, U>) -> TypedPoint3D<f32, U> {
    let l = length(v);
    TypedPoint3D::new(v.x / l,
                      v.y / l,
                      v.z / l)
}

fn shade(scene: &Scene,
         new_ray: &Ray,
         object: &Object) -> Color {
    const OBJECT_COLOR: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    const SPEC_COLOR: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    const AMBIENT_LIGHT_STRENGTH: f32 = 0.6;
    let ambient = OBJECT_COLOR * AMBIENT_LIGHT_STRENGTH; // Assume color = white

    let light_direction = normalize(scene.light_position - new_ray.position);

    // TODO(emilio): Don't recalculate this, get from ray.
    let normal = normalize(new_ray.position - object.position);
    let impact = normal.dot(light_direction).max(0.0);

    let diffuse = OBJECT_COLOR * impact;

    let reflection_direction = to_world_units(new_ray.direction);
    let spec = light_direction.dot(reflection_direction).max(0.0).powi(2);
    let specular = SPEC_COLOR * spec;

    return ambient + diffuse + specular;
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

        Color::new(slice[0] as f32 / 255.0,
                   slice[1] as f32 / 255.0,
                   slice[2] as f32 / 255.0,
                   slice[3] as f32 / 255.0)
    }


    fn set_color_at(&self,
                    framebuffer: &mut [u8],
                    x: usize,
                    y: usize,
                    color: Color) {
        let slice = &mut framebuffer[x * 4 + y * self.size.width * 4..];
        slice[0] = (color.r * 255.0).round().min(255.0) as u8;
        slice[1] = (color.g * 255.0).round().min(255.0) as u8;
        slice[2] = (color.b * 255.0).round().min(255.0) as u8;
        slice[3] = (color.a * 255.0).round().min(255.0) as u8;
    }

    fn set_z_at(&self,
                z_buffer: &mut [f32],
                x: usize,
                y: usize,
                val: f32) {
        let mut z_value = &mut z_buffer[x + self.size.width * y];
        *z_value = val;
    }

    fn get_z_at(&self,
                z_buffer: &[f32],
                x: usize,
                y: usize) -> f32 {
        z_buffer[x + self.size.width * y]
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

        // Sanity check.
        self.set_color_at(&mut framebuffer, self.size.width - 1, self.size.height - 1, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(framebuffer[framebuffer.len() - 1], 255);


        for x in 0..self.size.width {
            for y in 0..self.size.height {
                self.set_z_at(&mut z_buffer, x, y, ::std::f32::INFINITY);
                self.set_color_at(&mut framebuffer, x, y, Color::new(0.0, 0.0, 0.0, 1.0));
            }
        }

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

        // TODO(emilio): Allow other non-ortho perspectives so the camera
        // position is useful.
        let mut current_ray = Ray::new(
            WorldPosition::new(x as f32, y as f32, self.light_position.z),
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
                Some((current, object)) => {
                    let (x, y) = (current.position.x as usize,
                                  current.position.y as usize);

                    let z_value = self.get_z_at(z_buffer, x, y);
                    if current.position.z > z_value {
                        current_reflection += 1;
                        continue;
                    }
                    self.set_z_at(z_buffer, x, y,
                                  current.position.z.min(z_value));

                    // See if the current point is at shadow, if so, apply it.
                    let light_ray = Ray::new(self.light_position,
                                             normalize(current.position - self.light_position).to_untyped(),
                                             1.0);
                    let distance_to_light = length(light_ray.position - current.position);
                    let mut in_shadow = false;
                    for maybe_intersected_object in &self.objects {
                        if object as *const _ == maybe_intersected_object as *const _ {
                            continue; // Ignore the same object.
                        }
                        if let Some(intersection) = light_ray.intersects_with(&maybe_intersected_object) {
                            if length(intersection.position - light_ray.position) < distance_to_light {
                                in_shadow = true;
                                break;
                            }
                        }
                    }

                    let mut object_color = shade(self, &current, object);
                    if in_shadow {
                        object_color = object_color * 0.5;
                    }

                    // println!("Current: {:?}", current);

                    let color = self.get_color_at(framebuffer, x, y);
                    self.set_color_at(framebuffer, x, y,
                                      color + object_color * current_ray.strength);
                    current_ray = current;
                }
                None => break,
            }

            current_reflection += 1;
        }
    }
}
