use braille::{BrailleCharGridVector, BrailleCharTrait};

use std::ops::{Add, Mul};

use glam::{Vec3, Vec2, USizeVec2, Quat, EulerRot};


pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 1000.0;

#[inline(always)]
pub fn rotation_quat(yaw: f32, pitch: f32, roll: f32) -> Quat {
    return Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
}

#[inline(always)]
pub fn rotate_around(vertex: Vec3, pivot: Vec3, rotation: Quat) -> Vec3 {
    return rotation * (vertex - pivot) + pivot;
}

#[inline(always)]
pub fn project(vertex: Vec3) -> Vec2 {
    return Vec2::new(vertex.x / vertex.z, vertex.y / vertex.z);
}

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
        let forward = self.camera.rotation() * Vec3::Z;

        use std::io::{stdout, Write};
        use crossterm::{execute, cursor::MoveTo, style::Print};
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
            // if face.normal.dot(forward) >= 0.0 {
            //     continue;
            // }
            let a = self.vertices[face.vertices.0];
            let b = self.vertices[face.vertices.1];
            let c = self.vertices[face.vertices.2];

            let t_min = a.z.min(b.z).min(c.z);
            let t_max = a.z.max(b.z).max(c.z);
            if t_max > NEAR && t_min < FAR {
                let a_proj = (a.x / a.z * sx + sx, a.y / a.z * sy + sy, (a.z - NEAR) / (FAR - NEAR));
                let b_proj = (b.x / b.z * sx + sx, b.y / b.z * sy + sy, (b.z - NEAR) / (FAR - NEAR));
                let c_proj = (c.x / c.z * sx + sx, c.y / c.z * sy + sy, (c.z - NEAR) / (FAR - NEAR));

                let (x0, y0, z0) = a_proj;
                let (x1, y1, z1) = b_proj;
                let (x2, y2, z2) = c_proj;

                // canva.draw_circle(x0 as usize, y0 as usize, 3, 255.0);
                // canva.draw_circle(x1 as usize, y1 as usize, 3, 255.0);
                // canva.draw_circle(x2 as usize, y2 as usize, 3, 255.0);

                let xmin = (x0.min(x1).min(x2).floor() as i32).clamp(0, width as i32 - 1);
                let xmax = (x0.max(x1).max(x2).ceil() as i32).clamp(0, width as i32 - 1);

                let ymin = (y0.min(y1).min(y2).floor() as i32).clamp(0, height as i32 - 1);
                let ymax = (y0.max(y1).max(y2).ceil() as i32).clamp(0, height as i32 - 1);

                for y in ymin..=ymax {
                    for x in xmin..=xmax {
                        let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
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

                let pos = alpha * a + beta * b + gamma * c;
                let color = face.color;
                let normal = face.normal;

                let result = self.calc_color(pos, color, normal);
                // let result = 255.0;

                canva.array[x + y * width] = result;
            }
        }
    }

    pub fn calc_color(&self, pos: Vec3, color: f32, normal: Vec3) -> f32 {
        return color * self.lights.iter()
            .map(|Light { pos: light_pos, intensity }| {
            let l = light_pos - pos;
            0.0_f32.max(normal.dot(l)) * intensity / l.length().powi(3)
        })
        .sum::<f32>();
    }
}

#[derive(Clone, Copy)]
pub struct Face {
    pub vertices: (usize, usize, usize),
    pub normal: Vec3,
    pub color: f32
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
        return Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, self.roll);
    }
}

pub struct Light {
    pub pos: Vec3,
    pub intensity: f32
}

#[inline(always)]
pub fn to_screen(point: Vec2, width: f32, height: f32) -> Vec2 {
    return Vec2::new((point.x + 1.0) * 0.5 * width, (1.0 - point.y) * 0.5 * height);
}

#[inline(always)]
pub fn to_canva(point: Vec2, width: usize, height: usize) -> USizeVec2 {
    return to_screen(point, width as f32, height as f32).as_usizevec2();
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

