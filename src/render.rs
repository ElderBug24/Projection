use std::ops::{Add, Mul};

use glam::{Vec3, Vec2, Quat};
use image::{RgbImage, Rgb};


pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 1000.0;


pub struct Scene3D {
    pub camera: Camera,
    pub vertices: Vec<Vec3>,
    pub uv: Vec<Vec2>,
    pub faces: Vec<Face>,
    pub lights: Vec<Light>,
    pub textures: Vec<RgbImage>
}

impl Scene3D {
    pub fn render(&self, canva: &mut Canva) {
        let mut buffer: Vec<(usize, f32, (f32, f32, f32))> = vec![(0, f32::INFINITY, (0.0, 0.0, 0.0)); canva.width() * canva.height()];
        let width = canva.width();
        let height = canva.height();
        let sx = width as f32 / 2.0;
        let sy = height as f32 / 2.0;
        let fov = (self.camera.fov * 0.5).tan().recip();
        let aspect = width as f32 / height as f32;
        let camera_rotation = self.camera.rotation();
        let camera_pos = self.camera.pos;
        // let lights: Vec<Light> = self.lights.iter()
        //     .map(|Light { pos, intensity }| Light {
        //         pos: camera_rotation * (pos - camera_pos),
        //         intensity: *intensity
        //     }).collect();
        let mut faces: Vec<FaceOwned> = Vec::with_capacity(self.faces.len() * 2);

        for face in self.faces.iter() {
            let a = self.vertices[face.vertices.0];
            let b = self.vertices[face.vertices.1];
            let c = self.vertices[face.vertices.2];
            let (uv_a, uv_b, uv_c) = face.uv;

            let centroid = (a + b + c) / 3.0;
            if face.normal.dot(centroid - camera_pos) <= 0.0 {
                continue;
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
                    let uv_a = self.uv[uv_a];
                    let uv_b = self.uv[uv_b];
                    let uv_c = self.uv[uv_c];

                    faces.push(
                        FaceOwned {
                            vertices: (a, b, c),
                            uv: (uv_a, uv_b, uv_c),
                            normal: camera_rotation * face.normal,
                            texture_id: face.texture_id
                        }
                    );
                },
                1 => {
                    let ((a, b, c), (uv_a, uv_b, uv_c)) = match (a_, b_, c_) {
                        (true, false, false) => ((b, c, a), (uv_b, uv_c, uv_a)),
                        (false, true, false) => ((c, a, b), (uv_c, uv_a, uv_b)),
                        (false, false, true) => ((a, b, c), (uv_a, uv_b, uv_c)),
                        _ => unreachable!()
                    };

                    let uv_a = self.uv[uv_a];
                    let uv_b = self.uv[uv_b];
                    let uv_c = self.uv[uv_c];

                    let i = (NEAR - a.z) / (c.z - a.z);
                    let j = (NEAR - b.z) / (c.z - b.z);

                    let ac = c - a;
                    let bc = c - b;

                    let uv_ac = uv_c - uv_a;
                    let uv_bc = uv_c - uv_b;

                    let d = a + i * ac;
                    let e = b + j * bc;

                    let uv_d = uv_a + i * uv_ac;
                    let uv_e = uv_b + j * uv_bc;

                    let normal = camera_rotation * face.normal;
                    faces.push(
                        FaceOwned {
                            vertices: (a, b, e),
                            uv: (uv_a, uv_b, uv_e),
                            normal: normal,
                            texture_id: face.texture_id
                        }
                    );
                    faces.push(
                        FaceOwned {
                            vertices: (e, d, a),
                            uv: (uv_e, uv_d, uv_a),
                            normal: normal,
                            texture_id: face.texture_id
                        }
                    );
                },
                2 => {
                    let ((a, b, c), (uv_a, uv_b, uv_c)) = match (a_, b_, c_) {
                        (false, true, true) => ((a, b, c), (uv_a, uv_b, uv_c)),
                        (true, false, true) => ((b, c, a), (uv_b, uv_c, uv_a)),
                        (true, true, false) => ((c, a, b), (uv_c, uv_a, uv_b)),
                        _ => unreachable!()
                    };

                    let uv_a = self.uv[uv_a];
                    let uv_b = self.uv[uv_b];
                    let uv_c = self.uv[uv_c];

                    let i = (NEAR - a.z) / (b.z - a.z);
                    let j = (NEAR - a.z) / (c.z - a.z);

                    let ac = c - a;
                    let ab = b - a;

                    let uv_ac = uv_c - uv_a;
                    let uv_ab = uv_b - uv_a;

                    let d = a + j * ac;
                    let e = a + i * ab;

                    let uv_d = uv_a + j * uv_ac;
                    let uv_e = uv_a + i * uv_ab;

                    faces.push(
                        FaceOwned {
                            vertices: (a, e, d),
                            uv: (uv_a, uv_e, uv_d),
                            normal: camera_rotation * face.normal,
                            texture_id: face.texture_id
                        }
                    );
                },
                3 => {},
                _ => unreachable!()
            }
        }

        for (face_index, face) in faces.iter().enumerate() {
            let (a, b, c) = face.vertices;

            let (x0, y0, z0) = (a.x / a.z * fov / aspect * sx + sx, - a.y / a.z * fov * sy + sy, (a.z - NEAR) / (FAR - NEAR));
            let (x1, y1, z1) = (b.x / b.z * fov / aspect * sx + sx, - b.y / b.z * fov * sy + sy, (b.z - NEAR) / (FAR - NEAR));
            let (x2, y2, z2) = (c.x / c.z * fov / aspect * sx + sx, - c.y / c.z * fov * sy + sy, (c.z - NEAR) / (FAR - NEAR));

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

                        let index = x as usize + (height - 1 - y as usize) * width;
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
                let texture = &self.textures[face.texture_id];
                let (a, b, c) = face.vertices;
                let (uv_a, uv_b, uv_c) = face.uv;
                let (w_a, w_b, w_c) = (a.z.recip(), b.z.recip(), c.z.recip());

                let w = alpha * w_a + beta * w_b + gamma * w_c;
                let uv = (alpha * uv_a * w_a + beta * uv_b * w_b + gamma * uv_c * w_c) / w;
                let (u, v) = ((uv.x * texture.width() as f32) as u32, (uv.y * texture.height() as f32) as u32);
                let color = texture.get_pixel_checked(u, v).copied().unwrap_or(Rgb([0, 0, 0]));
                let [r, g, b] = color.0;
                let color = Vec3::new(r as f32, g as f32, b as f32);

                // let pos = alpha * a + beta * b + gamma * c;
                // let normal = face.normal;
                // let mut light_sum = 0.0;
                // for light in &lights {
                //     let l = light.pos - pos;
                //
                //     light_sum += 0.0_f32.max(normal.dot(l)) * light.intensity / l.length().powi(3);
                // }

                // let result = 255.0;
                let result = color;
                // let result = color * light_sum;

                canva.array[x + (height - y - 1) * width] = result;
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Face {
    pub vertices: (usize, usize, usize),
    pub uv: (usize, usize, usize),
    pub normal: Vec3,
    pub texture_id: usize
}

#[derive(Clone, Copy)]
pub struct FaceOwned {
    pub vertices: (Vec3, Vec3, Vec3),
    pub uv: (Vec2, Vec2, Vec2),
    pub normal: Vec3,
    pub texture_id: usize
}

pub fn new_face(scene: &mut Scene3D, vertices: (Vec3, Vec3, Vec3), texture_id: usize, uv: (Vec2, Vec2, Vec2)) {
    let (a, b, c) = vertices;
    let (uv_a, uv_b, uv_c) = uv;
    let normal = (b - a).cross(c - a);

    let index = scene.vertices.len();
    scene.vertices.reserve(3);
    scene.vertices.push(a);
    scene.vertices.push(b);
    scene.vertices.push(c);

    let index_uv = scene.uv.len();
    scene.uv.reserve(3);
    scene.uv.push(uv_a);
    scene.uv.push(uv_b);
    scene.uv.push(uv_c);

    scene.faces.push(
        Face {
            vertices: (index, index+1, index+2),
            uv: (index_uv, index_uv+1, index_uv+2),
            normal: normal,
            texture_id: texture_id
        }
    );
}

pub fn new_face_from_index(scene: &mut Scene3D, vertices: (usize, usize, usize), texture_id: usize, uv: (usize, usize, usize)) {
    let (a, b, c) = vertices;
    let va = scene.vertices[a];
    let vb = scene.vertices[b];
    let vc = scene.vertices[c];
    let normal = (vb - va).cross(vc - va);

    scene.faces.push(
        Face {
            vertices: (a, b, c),
            uv: uv,
            normal: normal,
            texture_id: texture_id
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

pub struct Light {
    pub pos: Vec3,
    pub intensity: f32
}

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

