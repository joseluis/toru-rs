use nalgebra as na;
use std::{fs, io, io::BufRead};

mod geom;
pub use geom::*;
mod cube;
pub use cube::*;
mod terrain;
pub use terrain::*;
mod material;
use crate::color::Color;
pub use material::*;

mod objfile {
	use nalgebra as na;

	pub fn parse_vertex(line: &str) -> na::Point3<f32> {
		let mut p = na::Point3::new(0.0, 0.0, 0.0);
		for (i, token) in line.split(' ').filter(|s| s != &"").skip(1).enumerate() {
			if let Ok(value) = token.parse::<f32>() {
				match i {
					0 => p.x = value,
					1 => p.y = value,
					2 => p.z = value,
					_ => {}
				}
			} else {
				// Invalid token: {:?}", token
			}
		}

		p
	}

	pub fn parse_face(line: &str) -> (usize, usize, usize) {
		let mut f = (0, 0, 0);
		for (i, token) in line.split(' ').filter(|s| s != &"").skip(1).enumerate() {
			let index = token.split('/').next().unwrap();
			if let Ok(value) = index.parse::<usize>() {
				match i {
					0 => f.2 = value - 1,
					1 => f.1 = value - 1,
					2 => f.0 = value - 1,
					_ => {}
				}
			} else {
				// Invalid token: {:?}", token
			}
		}

		f
	}
}

pub trait Mesh {
	fn transform(&self) -> Option<&na::Matrix4<f32>> {
		None
	}

	fn transform_mut(&mut self) -> Option<&mut na::Matrix4<f32>> {
		None
	}
	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle> + 'a>;
	fn len(&self) -> usize {
		0
	}
	fn color(&self) -> Option<Color> {
		None
	}
}

#[derive(Default, Clone)]
pub struct StaticMesh {
	pub transform: na::Matrix4<f32>,
	pub vertices: Vec<na::Point3<f32>>,
	pub normals: Vec<na::Vector3<f32>>,
	pub triangles: Vec<(usize, usize, usize)>,
	pub colors: Vec<Color>,
}

impl StaticMesh {
	pub fn new() -> Self {
		Self {
			transform: na::Matrix4::identity(),
			..Default::default()
		}
	}

	pub fn sphere(radius: f32, resolution: u8, color: Color) -> Self {
		let mut mesh = StaticMesh::new();
		let t = ((1.0 + 5.0f32.sqrt()) / 2.0);

		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(-1.0, t, 0.0).normalize()));
		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(1.0, t, 0.0).normalize()));
		mesh.vertices.push(na::Point3::from_coordinates(
			na::Vector3::new(-1.0, -t, 0.0).normalize(),
		));
		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(1.0, -t, 0.0).normalize()));

		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(0.0, -1.0, t).normalize()));
		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(0.0, 1.0, t).normalize()));
		mesh.vertices.push(na::Point3::from_coordinates(
			na::Vector3::new(0.0, -1.0, -t).normalize(),
		));
		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(0.0, 1.0, -t).normalize()));

		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(t, 0.0, -1.0).normalize()));
		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(t, 0.0, 1.0).normalize()));
		mesh.vertices.push(na::Point3::from_coordinates(
			na::Vector3::new(-t, 0.0, -1.0).normalize(),
		));
		mesh.vertices
			.push(na::Point3::from_coordinates(na::Vector3::new(-t, 0.0, 1.0).normalize()));

		let mut triangles = vec![];
		triangles.push((5, 11, 0));
		triangles.push((1, 5, 0));
		triangles.push((7, 1, 0));
		triangles.push((10, 7, 0));
		triangles.push((11, 10, 0));

		triangles.push((9, 5, 1));
		triangles.push((4, 11, 5));
		triangles.push((2, 10, 11));
		triangles.push((6, 7, 10));
		triangles.push((8, 1, 7));

		triangles.push((4, 9, 3));
		triangles.push((2, 4, 3));
		triangles.push((6, 2, 3));
		triangles.push((8, 6, 3));
		triangles.push((9, 8, 3));

		triangles.push((5, 9, 4));
		triangles.push((11, 4, 2));
		triangles.push((10, 2, 6));
		triangles.push((7, 6, 8));
		triangles.push((1, 8, 9));

		let mut get_mid = |p0: &na::Point3<f32>, p1: &na::Point3<f32>| -> na::Point3<f32> {
			let mut mid = na::Point3::from_coordinates((p0.coords + p1.coords).normalize());
			mid
		};

		let mut next_triangles = vec![];
		for _ in 0..resolution {
			for tri in triangles.drain(..) {
				let p0 = &mesh.vertices[tri.0].clone();
				let p1 = &mesh.vertices[tri.1].clone();
				let p2 = &mesh.vertices[tri.2].clone();
				mesh.vertices.push(get_mid(p0, p1));
				let a = mesh.vertices.len() - 1;
				mesh.vertices.push(get_mid(p1, p2));
				let b = mesh.vertices.len() - 1;
				mesh.vertices.push(get_mid(p2, p0));
				let c = mesh.vertices.len() - 1;

				next_triangles.push((tri.0, a, c));
				next_triangles.push((tri.1, b, a));
				next_triangles.push((tri.2, c, b));
				next_triangles.push((a, b, c));
			}
			std::mem::swap(&mut triangles, &mut next_triangles);
		}

		mesh.triangles = triangles;

		for t in &mut mesh.triangles {
			let p = mesh.vertices[t.0];
			let color = match (p.coords.norm() * 10.0) as i32 {
				-100..=100 => Color::green(),
				_ => Color::blue(),
			};
			mesh.colors.push(color);
		}

		for v in &mut mesh.vertices {
			v.coords *= radius;
		}

		mesh
	}

	pub fn load_obj(filename: &str) -> Result<Self, io::Error> {
		let mut obj = Self::new();
		let file = fs::File::open(filename)?;
		let lines = io::BufReader::new(file).lines();
		for line in lines {
			let line = line.unwrap();
			let leader = line.split(' ').next().unwrap_or("");
			match leader {
				// Vertex
				"v" => obj.vertices.push(objfile::parse_vertex(&line)),
				// Face
				"f" => obj.triangles.push(objfile::parse_face(&line)),
				// Vertex Texture
				"vt" => {}
				// Vertex Normal
				"vn" => {}
				_ => {}
			}
		}

		Ok(obj)
	}
}

impl Mesh for StaticMesh {
	fn transform(&self) -> Option<&na::Matrix4<f32>> {
		Some(&self.transform)
	}

	fn transform_mut(&mut self) -> Option<&mut na::Matrix4<f32>> {
		Some(&mut self.transform)
	}

	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle> + 'a> {
		Box::new(StaticMeshIterator::new(self))
	}

	fn len(&self) -> usize {
		StaticMeshIterator::new(self).len()
	}
}

pub struct StaticMeshIterator<'a> {
	current: usize,
	mesh: &'a StaticMesh,
}

impl<'a> StaticMeshIterator<'a> {
	pub fn new(mesh: &'a StaticMesh) -> Self {
		Self { current: 0, mesh }
	}
}

impl<'a> ExactSizeIterator for StaticMeshIterator<'a> {
	fn len(&self) -> usize {
		self.mesh.triangles.len()
	}
}

impl<'a> Iterator for StaticMeshIterator<'a> {
	type Item = Triangle;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}

		let tri = self.mesh.triangles[self.current];
		let mut tri = Triangle::new(
			self.mesh.vertices[tri.0].clone(),
			self.mesh.vertices[tri.1].clone(),
			self.mesh.vertices[tri.2].clone(),
		);
		let d = ((tri.points[0].coords + tri.points[1].coords + tri.points[2].coords) / 3.0)
			.norm()
			.abs() - 9.0;
		let color = if self.current < self.mesh.colors.len() {
			 self.mesh.colors[self.current]
		} else {
			Color::red()
		};

		tri.color = Some(color);
		self.current += 1;
		Some(tri)
	}
}
