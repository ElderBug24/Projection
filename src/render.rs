use crate::model::*;

use std::ops::{Add, Mul};
use std::rc::Rc;
use std::path::Path;

use glam::{Vec3, Vec2, Quat};
use image::{self, RgbImage, Rgb, ImageError};


pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 1000.0;
pub const CULLING: bool = true;


pub struct Scene3D {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub buffered_faces: Vec<FaceOwned>,
    pub buffered_materials: Vec<Material>,
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
            buffered_materials: Vec::new(),
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

#[derive(Clone, Copy, Debug)]
pub struct Face {
    pub vertices: (usize, usize, usize),
    pub uv: (usize, usize, usize),
    pub normals: (usize, usize, usize)
}

#[derive(Clone, Copy)]
pub struct FaceOwned {
    pub vertices: (Vec3, Vec3, Vec3),
    pub uv: (Vec2, Vec2, Vec2),
    pub normals: (Vec3, Vec3, Vec3),
    pub material_id: usize
}

impl Scene3D {
    pub fn queue_render(&mut self, model: &Model3D) {

        let camera_rotation = self.camera.rotation();
        let camera_pos = self.camera.pos;

        for Group { faces, material } in &model.groups {
            self.buffered_faces.reserve(faces.len());
            let material_id = self.buffered_materials.len();
            self.buffered_materials.push(material.clone());

            for face in faces.iter() {
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
                            FaceOwned {
                                vertices: (a, b, c),
                                uv: (uv_a, uv_b, uv_c),
                                normals: (n_a, n_b, n_c),
                                material_id: material_id
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
                            FaceOwned {
                                vertices: (a, b, e),
                                uv: (uv_a, uv_b, uv_e),
                                normals: (n_a, n_b, n_e),
                                material_id: material_id
                            }
                        );
                        self.buffered_faces.push(
                            FaceOwned {
                                vertices: (e, d, a),
                                uv: (uv_e, uv_d, uv_a),
                                normals: (n_e, n_d, n_a),
                                material_id: material_id
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
                            FaceOwned {
                                vertices: (a, e, d),
                                uv: (uv_a, uv_e, uv_d),
                                normals: (n_a, n_e, n_d),
                                material_id: material_id
                            }
                        );
                    },
                    _ => unreachable!()
                }
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
                let (face_index, z_pixel, (alpha, beta, gamma)) = self.pixel_buffer[x + y * width];
                if face_index == 0 {
                    continue;
                }

                let face = self.buffered_faces[face_index-1];
                let material = self.buffered_materials[face.material_id].clone();
                let (a, b, c) = face.vertices;
                let (w_a_, w_b_, w_c_) = (a.z.recip(), b.z.recip(), c.z.recip());
                let (w_a, w_b, w_c) = (alpha * w_a_, beta * w_b_, gamma * w_c_);
                let (uv_a, uv_b, uv_c) = face.uv;
                let (n_a, n_b, n_c) = face.normals;

                let w = w_a + w_b + w_c;
                let uv = (uv_a * w_a + uv_b * w_b + uv_c * w_c) / w;
                let normal = ((n_a * w_a + n_b * w_b + n_c * w_c) / w).normalize();
                let pos = (a * w_a + b * w_b + c * w_c) / w;

                let fragment = Fragment {
                    world_pos: pos,
                    normal: normal,
                    uv: Some(uv),
                    screen_pos: Vec3::new(x as f32, y as f32, z_pixel),
                    color: Vec3::ONE
                };

                let result = material.render(&fragment, &lights, Vec3::ONE);

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

        for light in lights {
            let [x, y, z] = light.pos.to_array();
            let (x, y) = (x / z * fov / aspect * sx + sx, y / z * fov * sy + sy);//, (z - NEAR) / (FAR - NEAR));
            canva.draw_circle(x as usize, height - y as usize - 1, 3, Vec3::splat(3.0));
        }
    }

    pub fn clear_queue(&mut self) {
        self.buffered_faces.clear();
    }
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

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum IlluminationModel {
    Illum0 = 0,
    Illum1 = 1,
    Illum2 = 2
}

pub struct Fragment {
    pub world_pos: Vec3,
    pub normal: Vec3,
    pub uv: Option<Vec2>,
    pub screen_pos: Vec3,
    pub color: Vec3
}

#[derive(Clone, Debug)]
pub struct Material {
    pub ns: f32,
    pub ka: f32,
    pub kd: Vec3,
    pub ks: Vec3,
    pub ke: Vec3,
    pub illum: IlluminationModel,
    pub texture: Option<Rc<RgbImage>>
}

impl Default for Material {
    fn default() -> Self {
        return Self {
            ns: 0.0,
            ka: 1.0,
            kd: Vec3::ONE,
            ks: Vec3::ZERO,
            ke: Vec3::ZERO,
            illum: IlluminationModel::Illum0,
            texture: Some(RgbImage::from_raw(1, 1, vec![255, 255, 255]).unwrap().into())
        };
    }
}

impl Material {
    pub fn render(&self, fragment: &Fragment, ligths: &[Light], ia: Vec3) -> Vec3 {
        let &Fragment { world_pos, normal, uv, screen_pos: _, color: _ } = fragment;
        let &Material { ns, ka, kd, ks, ke, illum, ref texture } = self;

        let kd = if let (Some(texture), Some(uv)) = (texture.as_ref(), uv) {
            rgb_to_vec3(texture.get_pixel_checked((uv.x * texture.width() as f32) as u32, (uv.y * texture.height() as f32) as u32).copied().unwrap_or(Rgb([0, 0, 0])))
        } else {
            Vec3::ONE
        } * kd;

        return match illum {
            IlluminationModel::Illum0 => ke + kd,
            IlluminationModel::Illum1 => ke + ka * ia + kd * ligths.iter().map(|light| light.intensity * light.color * normal.dot((light.pos - world_pos).normalize()).max(0.0)).sum::<Vec3>(),
            IlluminationModel::Illum2 => ke + ka * ia + {
                let v = (-world_pos).normalize();

                ligths.iter().map(|light| {
                    let l = (light.pos - world_pos).normalize();
                    let r = (2.0 * normal.dot(l) * normal - l).normalize();

                    light.intensity * light.color * (kd * normal.dot(l).max(0.0) + ks * r.dot(v).max(0.0).powf(ns))
                })
                .sum::<Vec3>()
            }
        };
    }

    pub fn with_texture(mut self, texture: Rc<RgbImage>) -> Self {
        self.texture = Some(texture);

        return self;
    }

    pub fn with_open_texture<P: AsRef<Path>>(mut self, filename: P) -> Result<Self, ImageError> {
        return image::open(filename)
            .map(|image| {
                let image = image.into_rgb8();
                self.texture = Some(image.into());

                self
            });
    }
}

