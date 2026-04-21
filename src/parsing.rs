use crate::{Material, ColorMapSource, ColorMapDestination, BumpTexture, IlluminationModel, Model3DBuilder, Face, Group};

use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::Path;
use std::time::Instant;
use std::cmp::Ordering;
use std::collections::HashMap;

use glam::{Vec3, Vec2};
use image::ImageError;


impl Model3DBuilder {
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Option<Self> {
        let start = Instant::now();

        let path = std::path::absolute(&filename).ok()?;
        if !std::path::Path::exists(&path) {
            println!("Error: file doesnt exist");

            return None;
        }
        let parent = path.parent()?;

        let mut vertices = Vec::new();
        let mut uv = Vec::new();
        let mut normals = Vec::new();
        let mut groups = vec![Group::default()];
        let mut groups_material_name = vec![String::new()];
        let mut group_id = 0;

        let mut faces = Vec::new();
        let mut faces_without_normals = Vec::new();
        let mut total_faces = 0;

        let mut materials = HashMap::new();
        materials.insert(groups_material_name[0].clone(), Material {
            ka: Vec3::new(0.18, 0.41, 0.18),
            kd: Vec3::new(0.18, 0.41, 0.18),
            ks: Vec3::splat(0.8),
            illum: IlluminationModel::Illum2,
            ..Default::default()
        });

        let file = File::open(&filename).ok()?;
        let lines = BufReader::new(file).lines();

        for (row, line) in lines.map_while(Result::ok).enumerate() {
            let row = row + 1;

            let mut words = line.trim().split_whitespace();
            let Some(mode) = words.next() else { continue };

            let Some((mode, line)) = line.trim().split_once(' ') else { continue };

            match mode {
                "#" => println!("Comment at line {}: {}", row, line),
                "o" => println!("Object name at line {}: {}", row, line),
                "g" => println!("Group name at line {}: {}", row, line),
                "v" => {
                    let mut words = line.split_whitespace();
                    let (Some(a), Some(b), Some(c)) = (words.next(), words.next(), words.next())  else { println!("Error parsing line {}", row); continue };

                    if let (Ok(x), Ok(y), Ok(z)) = (a.parse::<f32>(), b.parse::<f32>(), c.parse::<f32>()) {
                        vertices.push(Vec3::new(x, y, z));
                    } else {
                        println!("Error parsing line {}", row);
                    };
                },
                "vt" => {
                    let mut words = line.split_whitespace();
                    let (Some(a), b) = (words.next(), words.next())  else { println!("Error parsing line {}", row); continue };

                    let x_ = a.parse::<f32>();
                    if let Ok(x) = x_ {
                        let Ok(y) = b.map(|v| v.parse::<f32>()).transpose() else { println!("Error parsing line {}", row); continue };

                        uv.push(Vec2::new(x, 1.0 - y.unwrap_or(0.0)));
                    };
                },
                "vn" => {
                    let mut words = line.split_whitespace();
                    let (Some(a), Some(b), Some(c)) = (words.next(), words.next(), words.next())  else { println!("Error parsing line {}", row); continue };

                    if let (Ok(x), Ok(y), Ok(z)) = (a.parse::<f32>(), b.parse::<f32>(), c.parse::<f32>()) {
                        normals.push(Vec3::new(x, y, z));
                    } else {
                        println!("Error parsing line {}", row);
                    };
                },
                "f" => {
                    let mut words = line.split_whitespace();
                    faces.clear();
                    let mut face_words = words.map(|word| parse_face_word(word));

                    let (Some(a), Some(mut b)) = (face_words.next().flatten(), face_words.next().flatten())  else { println!("Error parsing line {}", row); continue };

                    while let Some(c) = face_words.next() {
                        let Some(c) = c else {
                            break;
                        };

                        let (v_a, v_b, v_c) = (a.vertex, b.vertex, c.vertex);
                        let (uv_a, uv_b, uv_c) = (a.uv, b.uv, c.uv);
                        let (n_a, n_b, n_c) = (a.normal, b.normal, c.normal);

                        if n_a == 0 || n_b == 0 || n_c == 0 { faces_without_normals.push((group_id, groups[group_id].faces.len())) }

                        let v_o = vertices.len() as isize;
                        let uv_o = uv.len() as isize;
                        let n_o = normals.len() as isize;

                        let v_a = match v_a.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => v_a - 1, Ordering::Less => v_o + v_a } as usize;
                        let v_b = match v_b.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => v_b - 1, Ordering::Less => v_o + v_b } as usize;
                        let v_c = match v_c.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => v_c - 1, Ordering::Less => v_o + v_c } as usize;
                        let uv_a = match uv_a.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => uv_a, Ordering::Less => uv_o + uv_a } as usize;
                        let uv_b = match uv_b.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => uv_b, Ordering::Less => uv_o + uv_b } as usize;
                        let uv_c = match uv_c.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => uv_c, Ordering::Less => uv_o + uv_c } as usize;
                        let n_a = match n_a.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => n_a - 1, Ordering::Less => n_o + n_a } as usize;
                        let n_b = match n_b.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => n_b - 1, Ordering::Less => n_o + n_b } as usize;
                        let n_c = match n_c.cmp(&0) { Ordering::Equal => 0, Ordering::Greater => n_c - 1, Ordering::Less => n_o + n_c } as usize;

                        let uv = if uv_a == 0 || uv_b == 0 || uv_c == 0 {
                            None
                        } else {
                            Some((uv_a-1, uv_b-1, uv_c-1))
                        };

                        faces.push(Face {
                            vertices: (v_a, v_b, v_c),
                            uv: uv,
                            normals: (n_a, n_b, n_c)
                        });

                        b = c;
                    }

                    total_faces += faces.len();
                    groups[group_id].faces.extend_from_slice(&faces);
                },
                "usemtl" => {
                    let material_name = line.trim();

                    if let Some(index) = groups_material_name.iter().position(|s| s == material_name) {
                        group_id = index;

                        continue
                    }

                    if groups[group_id].faces.len() == 0 {
                        let _ = groups.pop();
                        let name = groups_material_name.pop().unwrap();
                        materials.remove(&name).unwrap();
                    }

                    println!("New group with material at line {}: {}", row, material_name);

                    groups_material_name.push(material_name.to_string());
                    groups.push(Group::default());
                    let group_id = groups.len() - 1;
                }
                "mtllib" => {
                    let filename = line.trim();

                    println!("Loading mltlib '{}'", filename);

                    let Some(_) = parse_mtllib(parent.join(filename), &mut materials) else { println!("Error parsing line {}", row); continue };
                },
                "s" => {},
                _ => println!("Unknown at line {}: {} {}", row, mode, line)
            }
        }

        assert_eq!(0, faces_without_normals.len(), "Need to implement computing of missing normals");
        assert!(materials.len() >= groups.len());

        for (group_id, material_name) in groups_material_name.into_iter().enumerate() {
            let material = materials.get(&material_name).cloned();

            if let Some(material) = material {
                groups[group_id].material = material;
            } else {
                println!("Error: could not find material '{}'", material_name);
            }
        }

        println!("\nLoaded model in {:.2} secs", start.elapsed().as_secs_f32());
        println!(" - {} vertices", vertices.len());
        println!(" - {} uv", uv.len());
        println!(" - {} normals", normals.len());
        println!(" - {} faces", total_faces);
        println!(" - {} materials", materials.len());

        return Some(Self {
            vertices: vertices,
            uv: uv,
            normals: normals,
            groups: groups,
            ..Default::default()
        });
    }
}

#[derive(Debug)]
struct FaceWord {
    pub vertex: isize,
    pub uv: isize,
    pub normal: isize
}

fn parse_color<'a>(mut words: impl Iterator<Item = &'a str>) -> Option<Vec3> {
    let Some(a) = words.next() else { return None; };

    let (r, g, b) = if a == "xyz" {
        let (x, y, z) = if let (Some(b), Some(c)) = (words.next(), words.next()) {
            let (Ok(x), Ok(y), Ok(z)) = (a.parse::<f32>(), b.parse::<f32>(), c.parse::<f32>()) else { return None; };

            (x, y, z)
        } else {
            let l = a.parse::<f32>().ok()?;

            (l, l, l)
        };

        let r =  3.2404542*x - 1.5371385*y - 0.4985314*z;
        let g = -0.9692660*x + 1.8760108*y + 0.0415560*z;
        let b =  0.0556434*x - 0.2040259*y + 1.0572252*z;

        (r, g, b)
    } else if a == "spectral" {
        return None;
    } else {
        if let (Some(b), Some(c)) = (words.next(), words.next()) {
            let (Ok(r), Ok(g), Ok(b)) = (a.parse::<f32>(), b.parse::<f32>(), c.parse::<f32>()) else { return None; };

            (r, g, b)
        } else {
            let l = a.parse::<f32>().ok()?;

            (l, l, l)
        }
    };

    let color = Vec3::new(r, g, b);

    return Some(color);
}

fn parse_mtllib<P: AsRef<Path>>(filename: P, materials: &mut HashMap<String, Material>) -> Option<()> {
    let path = std::path::absolute(&filename).ok()?;
    if !std::path::Path::exists(&path) {
        println!("Error: mtllib file doesnt exist");

        return None;
    }
    let parent = path.parent()?;

    let file = File::open(&filename).ok()?;
    let lines = BufReader::new(file).lines();

    let mut material = None;
    let mut material_name = String::new();

    for (row, line) in lines.map_while(Result::ok).enumerate() {
        let row = row + 1;

        let Some((mode, line)) = line.trim().split_once(' ') else { continue };

        match mode {
            "newmtl" => {
                let name = line;
                println!("New material: '{}'", name);

                if let Some(material) = material {
                    materials.insert(material_name, material);
                }

                material = Some(Material::default());
                material_name = name.into();
            },
            "Ns" => {
                let Some(ns) = line.parse::<f32>().ok() else { println!("Error parsing line {}", row); continue };

                if let Some(material) = material.as_mut() {
                    material.ns = ns;
                }
            },
            "Ka" => {
                let Some(color) = parse_color(line.split_whitespace()) else { println!("Error parsing line {}", row); continue };

                if let Some(material) = material.as_mut() {
                    material.ka = color;
                }
            },
            "Kd" => {
                let Some(color) = parse_color(line.split_whitespace()) else { println!("Error parsing line {}", row); continue };

                if let Some(material) = material.as_mut() {
                    material.kd = color;
                }
            },
            "Ks" => {
                let Some(color) = parse_color(line.split_whitespace()) else { println!("Error parsing line {}", row); continue };

                if let Some(material) = material.as_mut() {
                    material.ks = color;
                }
            },
            "Ke" => {
                let Some(color) = parse_color(line.split_whitespace()) else { println!("Error parsing line {}", row); continue };

                if let Some(material) = material.as_mut() {
                    material.ke = color;
                }
            },
            "illum" => {
                let Some((model_id, _)) = line.split_once(' ') else { println!("Error parsing line {}", row); continue };

                if let Some(model) = model_id.parse::<u8>().ok().map(|value| value.min(2).try_into().ok()).flatten() {
                    if let Some(material) = material.as_mut() {
                        material.illum = model;
                    }
                }
            },
            "map_Ka" => {
                if let Some(material) = material.as_mut() {
                    if let Ok(color_map_source) = ColorMapSource::from_file_rgb(parent.join(line)) {
                        material.set_map(color_map_source, ColorMapDestination::Ka)
                    } else { println!("Error parsing line {}", row); continue }
                }
            },
            "map_Kd" => {
                if let Some(material) = material.as_mut() {
                    if let Ok(color_map_source) = ColorMapSource::from_file_rgb(parent.join(line)) {
                        material.set_map(color_map_source, ColorMapDestination::Kd)
                    } else { println!("Error parsing line {}", row); continue }
                }
            },
            "map_Ks" => {
                if let Some(material) = material.as_mut() {
                    if let Ok(color_map_source) = ColorMapSource::from_file_rgb(parent.join(line)) {
                        material.set_map(color_map_source, ColorMapDestination::Ks)
                    } else { println!("Error parsing line {}", row); continue }
                }
            },
            "map_Ns" => {
                if let Some(material) = material.as_mut() {
                    if let Ok(color_map_source) = ColorMapSource::from_file_l(parent.join(line)) {
                        material.set_map(color_map_source, ColorMapDestination::Ns)
                    } else { println!("Error parsing line {}", row); continue }
                }
            },
            "bump" | "map_bump" => {
                let bm: Option<(f32, &str)> = line.split_once(' ').filter(|(opt, rest)| *opt == "-bm").map(|(_, rest)| rest.split_once(' ').map(|(mult, rest)| mult.parse::<f32>().ok().map(|mult| (mult, rest))).flatten()).flatten();
                let (bm, filename) = bm.unwrap_or((1.0, line));

                if let Some(material) = material.as_mut() {
                    if let Ok(bump_map) = BumpTexture::from_file(filename, bm) {
                        material.map_bump = bump_map;
                    } else { println!("Error parsing line {}", row); continue }
                }
            },
            mode @ ("Ni" | "sharpness" | "d" | "map_d" | "Tr") => println!("Error parsing line {}, '{}' not supported", row, mode),
            mode @ _ => println!("Error parsing line {}, '{}' unknown", row, mode)
        }
    }

    if let Some(material) = material {
        materials.insert(material_name, material);
    }

    return Some(());
}

fn parse_face_word(word: &str) -> Option<FaceWord> {
    let mut parts = word.splitn(3, '/').map(|s| Some(s).filter(|s| !s.is_empty()));

    return Some(FaceWord {
        vertex: parts.next()??.parse::<isize>().ok()?,
        uv: parts.next().flatten().map(|s| s.parse::<isize>()).transpose().ok()?.unwrap_or(0),
        normal: parts.next().flatten().map(|s| s.parse::<isize>()).transpose().ok()?.unwrap_or(0),
    });
}

