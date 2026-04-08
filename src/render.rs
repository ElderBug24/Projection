use braille::{BrailleCharGridVector, BrailleCharTrait};

use std::ops::{Add, Mul};

use glam::{Vec3, Vec2, USizeVec2, Quat, EulerRot};
use std::io::{stdout, Write};
use crossterm::{execute, cursor::MoveTo, style::Print};


pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 1000.0;


pub struct Scene3D {
    pub camera: Camera,
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Face>,
    pub lights: Vec<Light>
}

impl Scene3D {
    pub fn render(&self, canva: &mut Canva) {
        let mut buffer: Vec<(usize, f32, (f32, f32, f32))> = vec![(0, f32::INFINITY, (0.0, 0.0, 0.0)); canva.width() * canva.height()];
        let width = canva.width();
        let height = canva.height();
        let sx = width as f32 / 2.0;
        let sy = height as f32 / 2.0;
        let camera_rotation = self.camera.rotation();
        let camera_pos = self.camera.pos;
        let lights: Vec<Light> = self.lights.iter()
        .map(|Light { pos, intensity }| Light {
            pos: camera_rotation * (pos - camera_pos),
            intensity: *intensity
        })
        .collect();
        let mut faces: Vec<FaceOwned> = Vec::with_capacity(self.faces.len()); // faceowned has vertices in camera space and inside all the faces are (culled?) .. they are not behind the camera at least

        for face in self.faces.iter() {
            let a = camera_rotation * (self.vertices[face.vertices.0] - camera_pos);
            let b = camera_rotation * (self.vertices[face.vertices.1] - camera_pos);
            let c = camera_rotation * (self.vertices[face.vertices.2] - camera_pos);

            let (a_, b_, c_) = (a.z <= NEAR, b.z <= NEAR, c.z <= NEAR);
            match a_ as u8 + b_ as u8 + c_ as u8 {
                0 => faces.push(
                    FaceOwned {
                        vertices: (a, b, c),
                        normal: camera_rotation * face.normal,
                        color: face.color
                    }
                ),
                1 => {
                    let (az, bz, cz) = match (a_, b_, c_) {
                        (true, false, false) => (b.z, c.z, a.z),
                        (false, true, false) => (c.z, a.z, b.z),
                        (false, false, true) => (a.z, b.z, c.z),
                        _ => unreachable!()
                    };

                    let i = (NEAR - az) / (cz - az);
                    let j = (NEAR - bz) / (cz - bz);

                    let ac = c - a;
                    let bc = b - a;

                    let u = a + i * ac;
                    let v = b + j * bc;

                    let normal = camera_rotation * face.normal;
                    faces.push(
                        FaceOwned {
                            vertices: (a, b, v),
                            normal: normal,
                            color: face.color
                        }
                    );
                    faces.push(
                        FaceOwned {
                            vertices: (v, u, a),
                            normal: normal,
                            color: face.color
                        }
                    );
                },
                2 => {
                    let (az, bz, cz) = match (a_, b_, c_) {
                        (false, true, true) => (a.z, b.z, c.z),
                        (true, false, true) => (b.z, c.z, a.z),
                        (true, true, false) => (c.z, a.z, b.z),
                        _ => unreachable!()
                    };

                    let i = (NEAR - az) / (bz - az);
                    let j = (NEAR - az) / (cz - az);

                    let ac = c - a;
                    let ab = b - a;

                    let u = a + j * ac;
                    let v = a + i * ab;

                    faces.push(
                        FaceOwned {
                            vertices: (a, v, u),
                            normal: camera_rotation * face.normal,
                            color: face.color
                        }
                    );
                },
                3 => {},
                _ => unreachable!()
            }
        }

        for (face_index, face) in self.faces.iter().enumerate() {
            // execute!(
            //     stdout(),
            //     MoveTo(0, 0),
            //     Print(face.normal.dot(forward)),
            //     MoveTo(0, 1),
            //     Print(forward),
            //     MoveTo(0, 2),
            //     Print(face.normal)
            // );
            // if (camera_rotation * face.normal).dot(camera_rotation * (self.vertices[face.vertices.0] - camera_pos)) >= 0.0 {
            //     continue;
            // }
            // if face.normal.dot(self.vertices[face.vertices.0] - camera_pos) >= 0.0 {
            //     continue;
            // }

            let a = camera_rotation * (self.vertices[face.vertices.0] - camera_pos);
            let b = camera_rotation * (self.vertices[face.vertices.1] - camera_pos);
            let c = camera_rotation * (self.vertices[face.vertices.2] - camera_pos);

            let t_min = a.z.min(b.z).min(c.z);
            // let t_max = a.z.max(b.z).max(c.z);
            if t_min > FAR || a.z <= NEAR || b.z <= NEAR || c.z <= NEAR {
                continue;
            }
            let a_proj = (a.x / a.z * sx + sx, a.y / a.z * sy + sy, (a.z - NEAR) / (FAR - NEAR));
            let b_proj = (b.x / b.z * sx + sx, b.y / b.z * sy + sy, (b.z - NEAR) / (FAR - NEAR));
            let c_proj = (c.x / c.z * sx + sx, c.y / c.z * sy + sy, (c.z - NEAR) / (FAR - NEAR));

            let (x0, y0, z0) = a_proj;
            let (x1, y1, z1) = b_proj;
            let (x2, y2, z2) = c_proj;

            canva.draw_circle(x0 as usize, y0 as usize, 3, 255.0);
            canva.draw_circle(x1 as usize, y1 as usize, 3, 255.0);
            canva.draw_circle(x2 as usize, y2 as usize, 3, 255.0);

            let xmin = (x0.min(x1).min(x2).floor() as i32).clamp(0, width as i32 - 1);
            let xmax = (x0.max(x1).max(x2).ceil() as i32).clamp(0, width as i32 - 1);

            let ymin = (y0.min(y1).min(y2).floor() as i32).clamp(0, height as i32 - 1);
            let ymax = (y0.max(y1).max(y2).ceil() as i32).clamp(0, height as i32 - 1);

            for y in ymin..=ymax {
                for x in xmin..=xmax {
                    let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
                    if denom.abs() < 1e-6 {
                        continue;
                    }
                    let denom_recip = denom.recip();

                    let alpha = ((y1 - y2) * (x as f32 - x2) + (x2 - x1) * (y as f32 - y2)) * denom_recip;
                    let beta  = ((y2 - y0) * (x as f32 - x2) + (x0 - x2) * (y as f32 - y2)) * denom_recip;
                    let gamma = 1.0 - alpha - beta;

                    if alpha >= 0.0 && beta >= 0.0 && gamma >= 0.0 {
                        let z_pixel = alpha * z0 + beta * z1 + gamma * z2;

                        let index = x as usize + y as usize * width;
                        if z_pixel < buffer[index].1 {
                            buffer[index] = (face_index + 1, z_pixel, (alpha, beta, gamma));
                        }
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let (face_index, z_pixel, (alpha, beta, gamma)) = buffer[x + y * width];
                if face_index == 0 {
                    continue;
                }

                let face = self.faces[face_index-1];
                let (a, b, c) = face.vertices;
                let (a, b, c) = (self.vertices[a], self.vertices[b], self.vertices[c]);
                let a = camera_rotation * (a - camera_pos);
                let b = camera_rotation * (b - camera_pos);
                let c = camera_rotation * (c - camera_pos);

                let pos = alpha * a + beta * b + gamma * c;
                let color = face.color;
                let normal = camera_rotation * face.normal;

                let mut light_sum = 0.0;
                for light in &lights {
                    let l = light.pos - pos;

                    light_sum += 0.0_f32.max(normal.dot(l)) * light.intensity / l.length().powi(3);
                }
                let result = color * light_sum;
                // let result = 255.0;

                canva.array[x + y * width] = result;
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Face {
    pub vertices: (usize, usize, usize),
    pub normal: Vec3,
    pub color: f32
}

#[derive(Clone, Copy)]
pub struct FaceOwned {
    pub vertices: (Vec3, Vec3, Vec3),
    pub normal: Vec3,
    pub color: f32
}

pub fn new_face(scene: &mut Scene3D, vertices: (Vec3, Vec3, Vec3), color: f32) {
    let (a, b, c) = vertices;
    let normal = (b - a).cross(c - a);

    let index = scene.vertices.len();
    scene.vertices.reserve(3);
    scene.vertices.push(a);
    scene.vertices.push(b);
    scene.vertices.push(c);

    scene.faces.push(
        Face {
            vertices: (index, index+1, index+2),
            normal: normal,
            color: color
        }
    );
}

pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32
}

impl Camera {
    pub fn new() -> Self {
        return Self {
            pos: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0
        };
    }

    pub fn rotation(&self) -> Quat {
        return Quat::from_rotation_x(self.pitch) * Quat::from_rotation_y(self.yaw);
    }

    pub fn forward(&self) -> Vec3 {
        return Vec3::new(-self.yaw.sin(), 0.0, self.yaw.cos());
    }

    pub fn right(&self) -> Vec3 {
        let forward = self.forward();
        return Vec3::new(forward.z, 0.0, -forward.x);
    }
}

pub struct Light {
    pub pos: Vec3,
    pub intensity: f32
}

pub struct Canva {
    pub array: Vec<f32>,
    width: usize,
    height: usize
}

impl Canva {
    #[inline(always)]
    pub fn new(width: usize, height: usize) -> Self {
        return Self {
            array: vec![0.0; width as usize * height as usize],
            width: width,
            height: height
        };
    }

    #[inline(always)]
    pub const fn width(&self) -> usize {
        return self.width;
    }

    #[inline(always)]
    pub const fn height(&self) -> usize {
        return self.height;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.array.fill(0.0);
    }

    #[inline(always)]
    pub fn resize(&mut self, width: usize, height: usize) {
        self.array.resize(width as usize * height as usize, 0.0);
        self.width = width;
        self.height = height;
    }

    pub fn draw_circle(&mut self, x: usize, y: usize, radius: usize, l: f32) {
        let xmin = x.saturating_sub(radius);
        let xmax = x.saturating_add(radius).min(self.width as usize);
        let ymin = y.saturating_sub(radius);
        let ymax = y.saturating_add(radius).min(self.height as usize);
        let r2 = radius.pow(2);

        for y_ in ymin..ymax {
            for x_ in xmin..xmax {
                let d2 = (x as isize - x_ as isize).pow(2) + (y as isize - y_ as isize).pow(2);

                if (d2 as usize) < r2 - 1 {
                    self.array[index(x_, y_, self.width)] = l;
                }
            }
        }
    }
}

#[inline(always)]
fn index<N: Add<Output = N> + Mul<Output = N>>(x: N, y: N, width: N) -> N {
    return x + y * width;
}

