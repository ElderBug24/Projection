use crate::model::*;

use std::ops::{Add, Mul};
use std::rc::Rc;

use glam::{Vec3, Vec2, Quat};
use image::{self, RgbImage, Rgb};


pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 1000.0;
pub const CULLING: bool = true;


pub struct Scene3D {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub buffered_faces: Vec<FaceOwned2>,
    pub buffered_textures: Vec<Rc<RgbImage>>,
    pub pixel_buffer: Vec<(usize, f32, (f32, f32, f32))>
}

#[derive(Default)]
pub struct Scene3DBuilder {
    pub camera: Camera,
    pub lights: Vec<Light>
}

impl Scene3DBuilder {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn build(self) -> Scene3D {
        return Scene3D {
            camera: self.camera,
            lights: self.lights,
            buffered_faces: Vec::new(),
            buffered_textures: Vec::new(),
            pixel_buffer: Vec::new()
        };
    }

    pub fn camera(mut self, camera: Camera) -> Self {
        self.camera = camera;

        return self;
    }

    pub fn lights(mut self, lights: &[Light]) -> Self {
        self.lights.extend(lights);

        return self;
    }
}

#[derive(Clone, Copy)]
pub struct Face {
    pub vertices: (usize, usize, usize),
    pub uv: (usize, usize, usize),
    pub normals: (usize, usize, usize)
}

#[derive(Clone, Copy)]
pub struct FaceOwned {
    pub vertices: (Vec3, Vec3, Vec3),
    pub uv: (Vec2, Vec2, Vec2),
    pub normals: (Vec3, Vec3, Vec3)
}

#[derive(Clone, Copy)]
pub struct FaceOwned2 {
    pub vertices: (Vec3, Vec3, Vec3),
    pub uv: (Vec2, Vec2, Vec2),
    pub normals: (Vec3, Vec3, Vec3),
    pub texture_id: usize
}

impl Scene3D {
    pub fn queue_render(&mut self, model: &Model3D) {
        self.buffered_faces.reserve_exact(model.faces.len() * 2);
        let texture_id = self.buffered_textures.len();
        self.buffered_textures.push(model.texture.clone());

        let camera_rotation = self.camera.rotation();
        let camera_pos = self.camera.pos;

        for face in model.faces.iter() {
            let a = model.vertices[face.vertices.0];
            let b = model.vertices[face.vertices.1];
            let c = model.vertices[face.vertices.2];
            let (uv_a, uv_b, uv_c) = face.uv;
            let (n_a, n_b, n_c) = face.normals;

            if CULLING {
                let centroid = (a + b + c) / 3.0;
                let normal = (b - a).cross(c - a);
                if normal.dot(centroid - camera_pos) >= 0.0 {
                    continue;
                }
            }

            let a = camera_rotation * (a - camera_pos);
            let b = camera_rotation * (b - camera_pos);
            let c = camera_rotation * (c - camera_pos);

            let (a_, b_, c_) = (a.z <= NEAR, b.z <= NEAR, c.z <= NEAR);

            if a_ && b_ && c_ {
                continue;
            }

            match a_ as u8 + b_ as u8 + c_ as u8 {
                0 => {
                    let uv_a = model.uv[uv_a];
                    let uv_b = model.uv[uv_b];
                    let uv_c = model.uv[uv_c];
                    let n_a = camera_rotation * model.normals[n_a];
                    let n_b = camera_rotation * model.normals[n_b];
                    let n_c = camera_rotation * model.normals[n_c];

                    self.buffered_faces.push(
                        FaceOwned2 {
                            vertices: (a, b, c),
                            uv: (uv_a, uv_b, uv_c),
                            normals: (n_a, n_b, n_c),
                            texture_id: texture_id
                        }
                    );
                },
                1 => {
                    let ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)) = match (a_, b_, c_) {
                        (true, false, false) => ((b, c, a), (uv_b, uv_c, uv_a), (n_b, n_c, n_a)),
                        (false, true, false) => ((c, a, b), (uv_c, uv_a, uv_b), (n_c, n_a, n_b)),
                        (false, false, true) => ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)),
                        _ => unreachable!()
                    };

                    let uv_a = model.uv[uv_a];
                    let uv_b = model.uv[uv_b];
                    let uv_c = model.uv[uv_c];
                    let n_a = camera_rotation * model.normals[n_a];
                    let n_b = camera_rotation * model.normals[n_b];
                    let n_c = camera_rotation * model.normals[n_c];

                    let i = (NEAR - a.z) / (c.z - a.z);
                    let j = (NEAR - b.z) / (c.z - b.z);

                    let ac = c - a;
                    let bc = c - b;
                    let uv_ac = uv_c - uv_a;
                    let uv_bc = uv_c - uv_b;
                    let n_ac = n_c - n_a;
                    let n_bc = n_c - n_b;

                    let d = a + i * ac;
                    let e = b + j * bc;
                    let uv_d = uv_a + i * uv_ac;
                    let uv_e = uv_b + j * uv_bc;
                    let n_d = n_a + i * n_ac;
                    let n_e = n_b + j * n_bc;

                    self.buffered_faces.push(
                        FaceOwned2 {
                            vertices: (a, b, e),
                            uv: (uv_a, uv_b, uv_e),
                            normals: (n_a, n_b, n_e),
                            texture_id: texture_id
                        }
                    );
                    self.buffered_faces.push(
                        FaceOwned2 {
                            vertices: (e, d, a),
                            uv: (uv_e, uv_d, uv_a),
                            normals: (n_e, n_d, n_a),
                            texture_id: texture_id
                        }
                    );
                },
                2 => {
                    let ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)) = match (a_, b_, c_) {
                        (false, true, true) => ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)),
                        (true, false, true) => ((b, c, a), (uv_b, uv_c, uv_a), (n_b, n_c, n_a)),
                        (true, true, false) => ((c, a, b), (uv_c, uv_a, uv_b), (n_c, n_a, n_b)),
                        _ => unreachable!()
                    };

                    let uv_a = model.uv[uv_a];
                    let uv_b = model.uv[uv_b];
                    let uv_c = model.uv[uv_c];
                    let n_a = camera_rotation * model.normals[n_a];
                    let n_b = camera_rotation * model.normals[n_b];
                    let n_c = camera_rotation * model.normals[n_c];

                    let i = (NEAR - a.z) / (b.z - a.z);
                    let j = (NEAR - a.z) / (c.z - a.z);

                    let ac = c - a;
                    let ab = b - a;
                    let uv_ac = uv_c - uv_a;
                    let uv_ab = uv_b - uv_a;
                    let n_ac = n_c - n_a;
                    let n_ab = n_b - n_a;

                    let d = a + j * ac;
                    let e = a + i * ab;
                    let uv_d = uv_a + j * uv_ac;
                    let uv_e = uv_a + i * uv_ab;
                    let n_d = n_a + j * n_ac;
                    let n_e = n_a + i * n_ab;

                    self.buffered_faces.push(
                        FaceOwned2 {
                            vertices: (a, e, d),
                            uv: (uv_a, uv_e, uv_d),
                            normals: (n_a, n_e, n_d),
                            texture_id: texture_id
                        }
                    );
                },
                _ => unreachable!()
            }
        }
    }

    pub fn render(&mut self, canva: &mut Canva) {
        self.pixel_buffer.clear();
        self.pixel_buffer.resize(canva.width() * canva.height(), (0, f32::INFINITY, (0.0, 0.0, 0.0)));
        let width = canva.width();
        let height = canva.height();
        let sx = width as f32 / 2.0;
        let sy = height as f32 / 2.0;
        let fov = (self.camera.fov * 0.5).tan().recip();
        let aspect = width as f32 / height as f32;
        let camera_rotation = self.camera.rotation();
        let camera_pos = self.camera.pos;
        let lights: Vec<Light> = self.lights.iter()
            .map(|Light { pos, intensity, color }| Light {
                pos: camera_rotation * (pos - camera_pos),
                intensity: *intensity,
                color: *color
            }).collect();

        for (face_index, face) in self.buffered_faces.iter().enumerate() {
            let (a, b, c) = face.vertices;

            let (x0, y0, z0) = (a.x / a.z * fov / aspect * sx + sx, a.y / a.z * fov * sy + sy, (a.z - NEAR) / (FAR - NEAR));
            let (x1, y1, z1) = (b.x / b.z * fov / aspect * sx + sx, b.y / b.z * fov * sy + sy, (b.z - NEAR) / (FAR - NEAR));
            let (x2, y2, z2) = (c.x / c.z * fov / aspect * sx + sx, c.y / c.z * fov * sy + sy, (c.z - NEAR) / (FAR - NEAR));

            // canva.draw_circle(x0 as usize, height - y0 as usize - 1, 3, Vec3::ONE);
            // canva.draw_circle(x1 as usize, height - y1 as usize - 1, 3, Vec3::ONE);
            // canva.draw_circle(x2 as usize, height - y2 as usize - 1, 3, Vec3::ONE);

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
                        if z_pixel < self.pixel_buffer[index].1 {
                            self.pixel_buffer[index] = (face_index + 1, z_pixel, (alpha, beta, gamma));
                        }
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let (face_index, _z_pixel, (alpha, beta, gamma)) = self.pixel_buffer[x + y * width];
                if face_index == 0 {
                    continue;
                }

                let face = self.buffered_faces[face_index-1];
                let texture = &self.buffered_textures[face.texture_id];
                let (a, b, c) = face.vertices;
                let (w_a, w_b, w_c) = (a.z.recip(), b.z.recip(), c.z.recip());
                let (uv_a, uv_b, uv_c) = face.uv;
                let (n_a, n_b, n_c) = face.normals;

                let w = alpha * w_a + beta * w_b + gamma * w_c;
                let uv = (alpha * uv_a * w_a + beta * uv_b * w_b + gamma * uv_c * w_c) / w;
                let normal = ((alpha * n_a * w_a + beta * n_b * w_b + gamma * n_c * w_c) / w).normalize();
                let (u, v) = ((uv.x * texture.width() as f32) as u32, (uv.y * texture.height() as f32) as u32);
                let color = rgb_to_vec3(texture.get_pixel_checked(u, v).copied().unwrap_or(Rgb([0, 0, 0])));
                let pos = (alpha * a * w_a + beta * b * w_b + gamma * c * w_c) / w;

                let fragment = Fragment {
                    pos: pos,
                    normal: normal,
                    color: color
                };

                let material = Material {
                    ns: 16.0,
                    ka: 0.1,
                    kd: 0.9,
                    ks: Vec3::new(0.0, 0.0, 0.0),
                    ke: Vec3::new(0.0, 0.0, 0.0),
                    illum: IlluminationModel::Illum3
                };

                let result = material.render(&fragment, &lights, 10.0);

                // let mut light_sum = 0.0;
                // for light in &lights {
                //     let l = light.pos - pos;
                //
                //     // light_sum += 0.0_f32.max(normal.dot(l)) * light.intensity / l.length().powi(3);
                //     light_sum += 0.1 + 0.9 * normal.dot(l.normalize()).abs() * light.intensity;
                // }
                //
                // // let light_sum = 1.0_f32;
                //
                // // let result = Vec3::ONE;
                // // let result = color;
                // let result = color * light_sum.clamp(0.0, 1.0);

                canva.array[x + (height - y - 1) * width] = result;
            }
        }
    }

    pub fn clear_queue(&mut self) {
        self.buffered_faces.clear();
    }

    pub fn _render_model(&self, model: &Model3D, canva: &mut Canva) {
        let mut buffer: Vec<(usize, f32, (f32, f32, f32))> = vec![(0, f32::INFINITY, (0.0, 0.0, 0.0)); canva.width() * canva.height()];
        let width = canva.width();
        let height = canva.height();
        let sx = width as f32 / 2.0;
        let sy = height as f32 / 2.0;
        let fov = (self.camera.fov * 0.5).tan().recip();
        let aspect = width as f32 / height as f32;
        let camera_rotation = self.camera.rotation();
        let camera_pos = self.camera.pos;
        let lights: Vec<Light> = self.lights.iter()
            .map(|Light { pos, intensity, color }| Light {
                pos: camera_rotation * (pos - camera_pos),
                intensity: *intensity,
                color: *color
            }).collect();
        let mut faces: Vec<FaceOwned> = Vec::with_capacity(model.faces.len() * 2);

        for face in model.faces.iter() {
            let a = model.vertices[face.vertices.0];
            let b = model.vertices[face.vertices.1];
            let c = model.vertices[face.vertices.2];
            let (uv_a, uv_b, uv_c) = face.uv;
            let (n_a, n_b, n_c) = face.normals;

            if CULLING {
                let centroid = (a + b + c) / 3.0;
                let normal = (b - a).cross(c - a);
                if normal.dot(centroid - camera_pos) <= 0.0 {
                    continue;
                }
            }

            let a = camera_rotation * (a - camera_pos);
            let b = camera_rotation * (b - camera_pos);
            let c = camera_rotation * (c - camera_pos);

            if a.z.min(b.z).min(c.z) > FAR {
                continue
            }

            let (a_, b_, c_) = (a.z <= NEAR, b.z <= NEAR, c.z <= NEAR);
            match a_ as u8 + b_ as u8 + c_ as u8 {
                0 => {
                    let uv_a = model.uv[uv_a];
                    let uv_b = model.uv[uv_b];
                    let uv_c = model.uv[uv_c];
                    let n_a = camera_rotation * model.normals[n_a];
                    let n_b = camera_rotation * model.normals[n_b];
                    let n_c = camera_rotation * model.normals[n_c];

                    faces.push(
                        FaceOwned {
                            vertices: (a, b, c),
                            uv: (uv_a, uv_b, uv_c),
                            normals: (n_a, n_b, n_c)
                        }
                    );
                },
                1 => {
                    let ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)) = match (a_, b_, c_) {
                        (true, false, false) => ((b, c, a), (uv_b, uv_c, uv_a), (n_b, n_c, n_a)),
                        (false, true, false) => ((c, a, b), (uv_c, uv_a, uv_b), (n_c, n_a, n_b)),
                        (false, false, true) => ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)),
                        _ => unreachable!()
                    };

                    let uv_a = model.uv[uv_a];
                    let uv_b = model.uv[uv_b];
                    let uv_c = model.uv[uv_c];
                    let n_a = camera_rotation * model.normals[n_a];
                    let n_b = camera_rotation * model.normals[n_b];
                    let n_c = camera_rotation * model.normals[n_c];

                    let i = (NEAR - a.z) / (c.z - a.z);
                    let j = (NEAR - b.z) / (c.z - b.z);

                    let ac = c - a;
                    let bc = c - b;
                    let uv_ac = uv_c - uv_a;
                    let uv_bc = uv_c - uv_b;
                    let n_ac = n_c - n_a;
                    let n_bc = n_c - n_b;

                    let d = a + i * ac;
                    let e = b + j * bc;
                    let uv_d = uv_a + i * uv_ac;
                    let uv_e = uv_b + j * uv_bc;
                    let n_d = n_a + i * n_ac;
                    let n_e = n_b + j * n_bc;

                    faces.push(
                        FaceOwned {
                            vertices: (a, b, e),
                            uv: (uv_a, uv_b, uv_e),
                            normals: (n_a, n_b, n_e)
                        }
                    );
                    faces.push(
                        FaceOwned {
                            vertices: (e, d, a),
                            uv: (uv_e, uv_d, uv_a),
                            normals: (n_e, n_d, n_a)
                        }
                    );
                },
                2 => {
                    let ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)) = match (a_, b_, c_) {
                        (false, true, true) => ((a, b, c), (uv_a, uv_b, uv_c), (n_a, n_b, n_c)),
                        (true, false, true) => ((b, c, a), (uv_b, uv_c, uv_a), (n_b, n_c, n_a)),
                        (true, true, false) => ((c, a, b), (uv_c, uv_a, uv_b), (n_c, n_a, n_b)),
                        _ => unreachable!()
                    };

                    let uv_a = model.uv[uv_a];
                    let uv_b = model.uv[uv_b];
                    let uv_c = model.uv[uv_c];
                    let n_a = camera_rotation * model.normals[n_a];
                    let n_b = camera_rotation * model.normals[n_b];
                    let n_c = camera_rotation * model.normals[n_c];

                    let i = (NEAR - a.z) / (b.z - a.z);
                    let j = (NEAR - a.z) / (c.z - a.z);

                    let ac = c - a;
                    let ab = b - a;
                    let uv_ac = uv_c - uv_a;
                    let uv_ab = uv_b - uv_a;
                    let n_ac = n_c - n_a;
                    let n_ab = n_b - n_a;

                    let d = a + j * ac;
                    let e = a + i * ab;
                    let uv_d = uv_a + j * uv_ac;
                    let uv_e = uv_a + i * uv_ab;
                    let n_d = n_a + j * n_ac;
                    let n_e = n_a + i * n_ab;

                    faces.push(
                        FaceOwned {
                            vertices: (a, e, d),
                            uv: (uv_a, uv_e, uv_d),
                            normals: (n_a, n_e, n_d)
                        }
                    );
                },
                _ => unreachable!()
            }
        }

        for (face_index, face) in faces.iter().enumerate() {
            let (a, b, c) = face.vertices;

            let (x0, y0, z0) = (a.x / a.z * fov / aspect * sx + sx, a.y / a.z * fov * sy + sy, (a.z - NEAR) / (FAR - NEAR));
            let (x1, y1, z1) = (b.x / b.z * fov / aspect * sx + sx, b.y / b.z * fov * sy + sy, (b.z - NEAR) / (FAR - NEAR));
            let (x2, y2, z2) = (c.x / c.z * fov / aspect * sx + sx, c.y / c.z * fov * sy + sy, (c.z - NEAR) / (FAR - NEAR));

            // canva.draw_circle(x0 as usize, y0 as usize, 3, Vec3::splat(255.0));
            // canva.draw_circle(x1 as usize, y1 as usize, 3, Vec3::splat(255.0));
            // canva.draw_circle(x2 as usize, y2 as usize, 3, Vec3::splat(255.0));

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

        for y in 0..height {
            for x in 0..width {
                let (face_index, _z_pixel, (alpha, beta, gamma)) = buffer[x + y * width];
                if face_index == 0 {
                    continue;
                }

                let face = faces[face_index-1];
                let texture = &model.texture;
                let (a, b, c) = face.vertices;
                let (w_a, w_b, w_c) = (a.z.recip(), b.z.recip(), c.z.recip());
                let (uv_a, uv_b, uv_c) = face.uv;
                let (n_a, n_b, n_c) = face.normals;

                let w = alpha * w_a + beta * w_b + gamma * w_c;
                let uv = (alpha * uv_a * w_a + beta * uv_b * w_b + gamma * uv_c * w_c) / w;
                let normal = ((alpha * n_a * w_a + beta * n_b * w_b + gamma * n_c * w_c) / w).normalize();
                let (u, v) = ((uv.x * texture.width() as f32) as u32, (uv.y * texture.height() as f32) as u32);
                let color = rgb_to_vec3(texture.get_pixel_checked(u, v).copied().unwrap_or(Rgb([0, 0, 0])));
                let pos = (alpha * a * w_a + beta * b * w_b + gamma * c * w_c) / w;

                let mut light_sum = 0.0;
                for light in &lights {
                    let l = light.pos - pos;

                    light_sum += 0.0_f32.max(normal.dot(l)) * light.intensity / l.length().powi(3);
                }
                let light_sum = 1.0_f32;

                // let result = Vec3::ONE;
                // let result = color;
                let result = color * light_sum.clamp(0.0, 1.0);

                canva.array[x + (height - y - 1) * width] = result;
            }
        }
    }
}

pub fn new_face(model: &mut Model3D, vertices: (Vec3, Vec3, Vec3), uv: (Vec2, Vec2, Vec2), normals: (Vec3, Vec3, Vec3)) {
    let (a, b, c) = vertices;
    let (uv_a, uv_b, uv_c) = uv;
    let (n_a, n_b, n_c) = normals;

    let index = model.vertices.len();
    model.vertices.reserve(3);
    model.vertices.push(a);
    model.vertices.push(b);
    model.vertices.push(c);

    let index_uv = model.uv.len();
    model.uv.reserve(3);
    model.uv.push(uv_a);
    model.uv.push(uv_b);
    model.uv.push(uv_c);

    let index_n = model.normals.len();
    model.normals.reserve(3);
    model.normals.push(n_a);
    model.normals.push(n_b);
    model.normals.push(n_c);

    model.faces.push(
        Face {
            vertices: (index, index+1, index+2),
            uv: (index_uv, index_uv+1, index_uv+2),
            normals: (index_n, index_n+1, index_n+2)
        }
    );
}

pub fn new_face_from_index(model: &mut Model3D, vertices: (usize, usize, usize), uv: (usize, usize, usize)) {
    let (a, b, c) = vertices;
    let va = model.vertices[a];
    let vb = model.vertices[b];
    let vc = model.vertices[c];
    let normal = -(vb - va).cross(vc - va).normalize();
    let index = model.normals.len();
    model.normals.push(normal);

    model.faces.push(
        Face {
            vertices: (a, b, c),
            uv: uv,
            normals: (index, index, index)
        }
    );
}

pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub fov: f32
}

impl Camera {
    pub fn rotation(&self) -> Quat {
        let pitch = Quat::from_rotation_x(self.pitch);
        let yaw = Quat::from_rotation_y(self.yaw);
        let roll = Quat::from_rotation_z(self.roll);

        return roll * pitch * yaw;
    }

    pub fn forward(&self) -> Vec3 {
        return Vec3::new(-self.yaw.sin(), 0.0, self.yaw.cos());
    }

    pub fn right(&self) -> Vec3 {
        let forward = self.forward();
        return Vec3::new(forward.z, 0.0, -forward.x);
    }
}

impl Default for Camera {
    fn default() -> Self {
        return Self {
            pos: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            fov: std::f32::consts::FRAC_PI_2
        };
    }
}

#[derive(Clone, Copy)]
pub struct Light {
    pub pos: Vec3,
    pub intensity: f32,
    pub color: Vec3
}

#[derive(Clone)]
pub struct Canva {
    pub array: Vec<Vec3>,
    width: usize,
    height: usize
}

impl Canva {
    #[inline(always)]
    pub fn new(width: usize, height: usize) -> Self {
        return Self {
            array: vec![Vec3::ZERO; width as usize * height as usize],
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
        self.array.fill(Vec3::ZERO);
    }

    #[inline(always)]
    pub fn resize(&mut self, width: usize, height: usize) {
        self.array.resize(width as usize * height as usize, Vec3::ZERO);
        self.width = width;
        self.height = height;
    }

    pub fn average_color(&self, x: usize, y: usize) -> Vec3 {
        let mut result = Vec3::ZERO;

        let (x, y) = (x * 2, y * 4);
        result += self.array[index(x, y, self.width)];
        result += self.array[index(x+1, y, self.width)];
        result += self.array[index(x, y+1, self.width)];
        result += self.array[index(x+1, y+1, self.width)];
        result += self.array[index(x, y+2, self.width)];
        result += self.array[index(x+1, y+2, self.width)];
        result += self.array[index(x, y+3, self.width)];
        result += self.array[index(x+1, y+3, self.width)];

        return result / 8.0;
    }

    pub fn draw_circle(&mut self, x: usize, y: usize, radius: usize, color: Vec3) {
        let xmin = x.saturating_sub(radius);
        let xmax = x.saturating_add(radius).min(self.width as usize);
        let ymin = y.saturating_sub(radius);
        let ymax = y.saturating_add(radius).min(self.height as usize);
        let r2 = radius.pow(2);

        for y_ in ymin..ymax {
            for x_ in xmin..xmax {
                let d2 = (x as isize - x_ as isize).pow(2) + (y as isize - y_ as isize).pow(2);

                if (d2 as usize) < r2 - 1 {
                    self.array[index(x_, y_, self.width)] = color;
                }
            }
        }
    }
}

#[inline(always)]
fn index<N: Add<Output = N> + Mul<Output = N>>(x: N, y: N, width: N) -> N {
    return x + y * width;
}

#[inline(always)]
pub fn rgb_to_vec3(rgb: Rgb<u8>) -> Vec3 {
    let [r, g, b] = rgb.0;
    const INV: f32 = 1.0 / 255.0;

    return Vec3::new(r as f32 * INV, g as f32 * INV, b as f32 * INV);
}

#[inline(always)]
pub fn vec3_to_rgb(v: Vec3) -> Rgb<u8> {
    let r = (v.x * 255.0).clamp(0.0, 255.0) as u8;
    let g = (v.y * 255.0).clamp(0.0, 255.0) as u8;
    let b = (v.z * 255.0).clamp(0.0, 255.0) as u8;

    return Rgb([r, g, b]);
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum IlluminationModel {
    Illum0 = 0,
    Illum1 = 1,
    Illum2 = 2,
    Illum3 = 3
}

impl IlluminationModel {
    pub fn into_u8(&self) -> u8 {
        return match self {
            Self::Illum0 => 0,
            Self::Illum1 => 1,
            Self::Illum2 => 2,
            Self::Illum3 => 3
        };
    }

    pub fn try_from_u8(value: u8) -> Option<Self> {
        return match value {
            0 => Some(Self::Illum0),
            1 => Some(Self::Illum1),
            2 => Some(Self::Illum2),
            3 => Some(Self::Illum3),
            _ => None
        };
    }
}

pub struct Fragment {
    pub pos: Vec3,
    pub normal: Vec3,
    pub color: Vec3
}

pub struct Material {
    pub ns: f32,
    pub ka: f32,
    pub kd: f32,
    pub ks: Vec3,
    pub ke: Vec3,
    pub illum: IlluminationModel
}

impl Material {
    pub fn render(&self, fragment: &Fragment, ligths: &[Light], la: f32) -> Vec3 {
        let &Fragment { pos, normal, color } = fragment;
        let &Material { ns, ka, mut kd, ks, ke, illum, .. } = self;

        // if let Some(color) = color {
        //     kd = color;
        // }

        return (color + ke) * match illum {
            IlluminationModel::Illum0 => Vec3::ZERO,
            IlluminationModel::Illum1 => Vec3::splat(ka * la),
            IlluminationModel::Illum2 => ka * la + kd * ligths.iter().map(|Light { pos: light_pos, intensity: light_intensity, color: light_color }| light_intensity * light_color * normal.dot((light_pos - pos).normalize()).max(0.0)).sum::<Vec3>(),
            IlluminationModel::Illum3 => {
                let v = (-pos).normalize();

                ka * la + ligths.iter().map(|Light { pos: light_pos, intensity: light_intensity, color: light_color }| {
                    let l = (light_pos - pos).normalize();
                    let h = (l + v).normalize();

                    light_intensity * light_color * (kd * normal.dot(l).max(0.0) + ks * normal.dot(h).max(0.0).powf(ns))
                })
                .sum::<Vec3>()
            }
        };
    }
}

