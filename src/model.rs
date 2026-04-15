use crate::render::*;

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::cmp::Ordering;
use std::time::Instant;

use glam::{Vec3, Vec2};
use image::RgbImage;


pub struct Model3D {
    pub vertices: Vec<Vec3>,
    pub uv: Vec<Vec2>,
    pub normals: Vec<Vec3>,
    pub groups: Vec<Group>
}

#[derive(Default)]
pub struct Model3DBuilder {
    pub vertices: Vec<Vec3>,
    pub uv: Vec<Vec2>,
    pub normals: Vec<Vec3>,
    pub groups: Vec<Group>
}

impl Model3DBuilder {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn build(self) -> Model3D {
        return Model3D {
            vertices: self.vertices,
            uv: self.uv,
            normals: self.normals,
            groups: self.groups
        };
    }

    pub fn from_file<P: AsRef<Path>>(filename: P) -> io::Result<Self> {
        let start = Instant::now();

        let mut vertices = Vec::new();
        let mut uv = Vec::new();
        let mut normals = Vec::new();
        let mut groups = vec![Group::default()];
        let mut group_id = 0;

        let mut faces = Vec::new();
        let mut faces_without_normals = Vec::new();
        let mut total_faces = 0;

        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        for (row, line) in lines.map_while(Result::ok).enumerate() {
            let row = row + 1;
            let mut words = line.split_whitespace();
            let Some(mode) = words.next() else {
                continue;
            };

            match mode {
                "#" => println!("Comment at line {}: {}", row, words.fold(String::new(), |a, b| a + b + " ")),
                "o" => println!("Object name at line {}: {}", row, words.fold(String::new(), |a, b| a + b + " ")),
                "g" => println!("Group name at line {}: {}", row, words.fold(String::new(), |a, b| a + b + " ")),
                "v" => {
                    let Some(a) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };
                    let Some(b) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };
                    let Some(c) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };

                    if let (Ok(x), Ok(y), Ok(z)) = (a.parse::<f32>(), b.parse::<f32>(), c.parse::<f32>()) {
                        vertices.push(Vec3::new(x, y, z));
                    } else {
                        println!("Error parsing line {}", row);
                    };
                },
                "vt" => {
                    let Some(a) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };
                    let Some(b) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };
                    let (x_, y_) = (a.parse::<f32>(), b.parse::<f32>());
                    if let Ok(x) = x_ {
                        let y = y_.unwrap_or(0.0);

                        uv.push(Vec2::new(x, y));
                    };
                },
                "vn" => {
                    let Some(a) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };
                    let Some(b) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };
                    let Some(c) = words.next() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };

                    if let (Ok(x), Ok(y), Ok(z)) = (a.parse::<f32>(), b.parse::<f32>(), c.parse::<f32>()) {
                        normals.push(Vec3::new(x, y, z));
                    } else {
                        println!("Error parsing line {}", row);
                    };
                },
                "f" => {
                    faces.clear();
                    let mut face_words = words.map(|word| parse_face_word(word));

                    let Some(a) = face_words.next().flatten() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };
                    let Some(mut b) = face_words.next().flatten() else {
                        println!("Error parsing line {}", row);

                        continue;
                    };

                    while let Some(c) = face_words.next() {
                        let Some(c) = c else {
                            break;
                        };

                        let (v_a, v_b, v_c) = (a.vertex, b.vertex, c.vertex);
                        let (uv_a, uv_b, uv_c) = (a.uv, b.uv, c.uv);
                        let (n_a, n_b, n_c) = (a.normal, b.normal, c.normal);

                        if uv_a == 0 || uv_b == 0 || uv_c == 0 {
                            println!("Error parsing line {}", row);

                            continue;
                        }
                        if n_a == 0 || n_b == 0 || n_c == 0 {
                            faces_without_normals.push((group_id, groups[group_id].faces.len()));
                        }

                        let v_o = vertices.len() as isize;
                        let uv_o = uv.len() as isize;
                        let n_o = normals.len() as isize;

                        let v_a = match v_a.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => v_a - 1, Ordering::Less => v_o + v_a } as usize;
                        let v_b = match v_b.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => v_b - 1, Ordering::Less => v_o + v_b } as usize;
                        let v_c = match v_c.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => v_c - 1, Ordering::Less => v_o + v_c } as usize;
                        let uv_a = match uv_a.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => uv_a - 1, Ordering::Less => uv_o + uv_a } as usize;
                        let uv_b = match uv_b.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => uv_b - 1, Ordering::Less => uv_o + uv_b } as usize;
                        let uv_c = match uv_c.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => uv_c - 1, Ordering::Less => uv_o + uv_c } as usize;
                        let n_a = match n_a.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => n_a - 1, Ordering::Less => n_o + n_a } as usize;
                        let n_b = match n_b.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => n_b - 1, Ordering::Less => n_o + n_b } as usize;
                        let n_c = match n_c.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => n_c - 1, Ordering::Less => n_o + n_c } as usize;

                        faces.push(Face {
                            vertices: (v_a, v_b, v_c),
                            uv: (uv_a, uv_b, uv_c),
                            normals: (n_a, n_b, n_c)
                        });

                        b = c;
                    }

                    total_faces += faces.len();
                    groups[group_id].faces.extend_from_slice(&faces);
                },
                _ => println!("Unknown at line {}: {} {}", row, mode, words.fold(String::new(), |a, b| a + b + " "))
            }
        }

        assert_eq!(0, faces_without_normals.len());

        println!("\nLoaded model in {:.2} secs", start.elapsed().as_secs_f32());
        println!(" - {} vertices", vertices.len());
        println!(" - {} uv", uv.len());
        println!(" - {} normals", normals.len());
        println!(" - {} faces", total_faces);

        groups[0].material = Material {
            ns: 32.0,
            ka: 0.2,
            kd: Vec3::splat(0.8),
            ks: Vec3::splat(0.8),
            ke: Vec3::ZERO,
            illum: IlluminationModel::Illum2,
            texture: Some(RgbImage::from_raw(1, 1, vec![47, 107, 47]).unwrap().into())
        };

        return Ok(Self {
            vertices: vertices,
            uv: uv,
            normals: normals,
            groups: groups,
            ..Default::default()
        });
    }

    pub fn vertices(mut self, vertices: &[Vec3]) -> Self {
        self.vertices.extend(vertices);

        return self;
    }

    pub fn uv(mut self, uv: &[Vec2]) -> Self {
        self.uv.extend(uv);

        return self;
    }

    pub fn normals(mut self, normals: &[Vec3]) -> Self {
        self.normals.extend(normals);

        return self;
    }

    pub fn faces(mut self, faces: &[Face], group_id: usize) -> Self {
        self.groups[group_id].faces.extend(faces);

        return self;
    }

    pub fn face_from_index(mut self, vertices: (usize, usize, usize), uv: (usize, usize, usize), group_id: usize) -> Self {
        let (a, b, c) = vertices;
        let va = self.vertices[a];
        let vb = self.vertices[b];
        let vc = self.vertices[c];
        let normal = -(vb - va).cross(vc - va).normalize();
        let index = self.normals.len();
        self.normals.push(normal);

        println!("{group_id}");
        println!("{:?}", self.groups);

        self.groups[group_id].faces.push(
            Face {
                vertices: (a, b, c),
                uv: uv,
                normals: (index, index, index)
            }
        );

        return self;
    }

    pub fn groups(mut self, groups: &[Group]) -> Self {
        self.groups.extend_from_slice(groups);

        return self;
    }

    pub fn material(mut self, material: Material, group_id: usize) -> Self {
        self.groups[group_id].material = material;

        return self;
    }
}

#[derive(Default, Clone, Debug)]
pub struct Group {
    pub faces: Vec<Face>,
    pub material: Material
}

#[derive(Debug)]
struct FaceWord {
    pub vertex: isize,
    pub uv: isize,
    pub normal: isize
}

fn parse_face_word(word: &str) -> Option<FaceWord> {
    let mut parts = word.splitn(3, '/').map(|s| Some(s).filter(|s| !s.is_empty()));

    return Some(FaceWord {
        vertex: parts.next()??.parse::<isize>().ok()?,
        uv: parts.next().flatten().map(|s| s.parse::<isize>()).transpose().ok()?.unwrap_or(0),
        normal: parts.next().flatten().map(|s| s.parse::<isize>()).transpose().ok()?.unwrap_or(0),
    });
}

