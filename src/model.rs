use crate::render::*;

use glam::{Vec3, Vec2};


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

    pub fn face_from_index(mut self, vertices: (usize, usize, usize), uv: Option<(usize, usize, usize)>, group_id: usize) -> Self {
        let (a, b, c) = vertices;
        let va = self.vertices[a];
        let vb = self.vertices[b];
        let vc = self.vertices[c];
        let normal = -(vb - va).cross(vc - va).normalize();
        let index = self.normals.len();
        self.normals.push(normal);

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

