use gdnative::{api::Mesh, prelude::*};

use super::chunk::*;

const CORNERS: [VPos; 8] = [
	(0, 1, 1),
	(1, 1, 1),
	(1, 1, 0),
	(0, 1, 0),
	(0, 0, 1),
	(1, 0, 1),
	(1, 0, 0),
	(0, 0, 0),
];

pub fn generate(chunks: ChunkBox2, offset: Vector3, surface_level: Voxel) -> Option<VariantArray> {
	let mut vertexes: PoolArray<Vector3> = PoolArray::new();
	let mut normals: PoolArray<Vector3> = PoolArray::new();

	let mut vert_space = 0usize;
	let mut vert_count = 0usize;

	for x in 0..WIDTH {
		for y in 0..WIDTH {
			for z in 0..WIDTH {
				let mut cube_state = 0u8;
				let mut cube_values = [0; 8];
				for i in 0..8 {
					let pos = (x as i8, y as i8, z as i8).add(CORNERS[i]);
					cube_values[i] = chunks.get(pos);
					cube_state |= ((cube_values[i] >= surface_level) as u8) << i;
				}

				// cube is entirely inside or outside the shape, so has no surface
				if cube_state == 0 || cube_state == 255 {
					continue;
				}

				let edge_mask = EDGEMASK[cube_state as usize];
				let mut vertlist = [Vector3::ZERO; 12];

				// Find the interpolated position of vertices for each edge
				for n in 0..12 {
					if edge_mask & (1 << n) != 0 {
						let corner_index_a = n % 8;
						let corner_index_b = [1, 2, 3, 0, 5, 6, 7, 4, 4, 5, 6, 7][n];
						let corner_a = CORNERS[corner_index_a];
						let corner_b = CORNERS[corner_index_b];
						let value_a = cube_values[corner_index_a];
						let value_b = cube_values[corner_index_b];
						vertlist[n] =
							interpolate_vert(surface_level, corner_a, corner_b, value_a, value_b)
					}
				}

				if vert_space - vert_count < 15 {
					vert_space += 45;
					vertexes.resize(vert_space as i32);
					normals.resize(vert_space as i32);
				}
				let mut vert_w = vertexes.write();
				let mut normals_w = normals.write();

				let triangles = TRIANGLES[cube_state as usize];
				let mut i = 0;
				let pos = (x as i8, y as i8, z as i8).vector() + offset;
				while triangles[i] != 255 {
					let vert_a = vertlist[triangles[i] as usize];
					let vert_b = vertlist[triangles[i + 1] as usize];
					let vert_c = vertlist[triangles[i + 2] as usize];
					vert_w[vert_count] = vert_c + pos;
					vert_w[vert_count + 1] = vert_b + pos;
					vert_w[vert_count + 2] = vert_a + pos;
					let normal = (vert_a - vert_c).cross(vert_b - vert_c);
					normals_w[vert_count] = normal;
					normals_w[vert_count + 1] = normal;
					normals_w[vert_count + 2] = normal;
					vert_count += 3;
					i += 3;
				}
			}
		}
	}

	if vert_count == 0 {
		return None;
	}

	vertexes.resize(vert_count as i32);
	normals.resize(vert_count as i32);

	let mesh_data = VariantArray::new_thread_local();
	mesh_data.resize(Mesh::ARRAY_MAX as i32);
	mesh_data.set(Mesh::ARRAY_VERTEX as i32, &vertexes);
	mesh_data.set(Mesh::ARRAY_NORMAL as i32, &normals);
	Some(unsafe { mesh_data.assume_unique().into_shared() })
}

fn interpolate_vert(
	surface_level: Voxel,
	corner_a: VPos,
	corner_b: VPos,
	value_a: Voxel,
	value_b: Voxel,
) -> Vector3 {
	// return (corner_a.vector() + corner_b.vector()) / 2.0;
	// if value_a == surface_level { return corner_a.vector() }
	// if value_b == surface_level { return corner_b.vector() }
	// if value_b == value_a { return corner_a.vector() }

	let surface_level = surface_level as f32 / 255.0;
	let value_a = value_a as f32 / 255.0;
	let value_b = value_b as f32 / 255.0;

	let delta = (surface_level - value_a) / (value_b - value_a);

	let pos_a = corner_a.vector();
	let pos_b = corner_b.vector();
	pos_a + delta * (pos_b - pos_a)
}

#[rustfmt::skip]
const EDGEMASK: [u16; 256]=[
	0x0  , 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c,
	0x80c, 0x905, 0xa0f, 0xb06, 0xc0a, 0xd03, 0xe09, 0xf00,
	0x190, 0x99 , 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c,
	0x99c, 0x895, 0xb9f, 0xa96, 0xd9a, 0xc93, 0xf99, 0xe90,
	0x230, 0x339, 0x33 , 0x13a, 0x636, 0x73f, 0x435, 0x53c,
	0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30,
	0x3a0, 0x2a9, 0x1a3, 0xaa , 0x7a6, 0x6af, 0x5a5, 0x4ac,
	0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0,
	0x460, 0x569, 0x663, 0x76a, 0x66 , 0x16f, 0x265, 0x36c,
	0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963, 0xa69, 0xb60,
	0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0xff , 0x3f5, 0x2fc,
	0xdfc, 0xcf5, 0xfff, 0xef6, 0x9fa, 0x8f3, 0xbf9, 0xaf0,
	0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x55 , 0x15c,
	0xe5c, 0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950,
	0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6, 0x2cf, 0x1c5, 0xcc ,
	0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0,
	0x8c0, 0x9c9, 0xac3, 0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc,
	0xcc , 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9, 0x7c0,
	0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c,
	0x15c, 0x55 , 0x35f, 0x256, 0x55a, 0x453, 0x759, 0x650,
	0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc,
	0x2fc, 0x3f5, 0xff , 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0,
	0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f, 0xd65, 0xc6c,
	0x36c, 0x265, 0x16f, 0x66 , 0x76a, 0x663, 0x569, 0x460,
	0xca0, 0xda9, 0xea3, 0xfaa, 0x8a6, 0x9af, 0xaa5, 0xbac,
	0x4ac, 0x5a5, 0x6af, 0x7a6, 0xaa , 0x1a3, 0x2a9, 0x3a0,
	0xd30, 0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c,
	0x53c, 0x435, 0x73f, 0x636, 0x13a, 0x33 , 0x339, 0x230,
	0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c,
	0x69c, 0x795, 0x49f, 0x596, 0x29a, 0x393, 0x99 , 0x190,
	0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905, 0x80c,
	0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x0
];

#[rustfmt::skip]
const TRIANGLES: [[u8; 16]; 256] = [
	[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 1, 9, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 8, 3, 9, 8, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 3, 1, 2, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[9, 2, 10, 0, 2, 9, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[2, 8, 3, 2, 10, 8, 10, 9, 8, 255, 255, 255, 255, 255, 255, 255],
	[3, 11, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 11, 2, 8, 11, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 9, 0, 2, 3, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 11, 2, 1, 9, 11, 9, 8, 11, 255, 255, 255, 255, 255, 255, 255],
	[3, 10, 1, 11, 10, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 10, 1, 0, 8, 10, 8, 11, 10, 255, 255, 255, 255, 255, 255, 255],
	[3, 9, 0, 3, 11, 9, 11, 10, 9, 255, 255, 255, 255, 255, 255, 255],
	[9, 8, 10, 10, 8, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 7, 8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 3, 0, 7, 3, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 1, 9, 8, 4, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 1, 9, 4, 7, 1, 7, 3, 1, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 10, 8, 4, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[3, 4, 7, 3, 0, 4, 1, 2, 10, 255, 255, 255, 255, 255, 255, 255],
	[9, 2, 10, 9, 0, 2, 8, 4, 7, 255, 255, 255, 255, 255, 255, 255],
	[2, 10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4, 255, 255, 255, 255],
	[8, 4, 7, 3, 11, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[11, 4, 7, 11, 2, 4, 2, 0, 4, 255, 255, 255, 255, 255, 255, 255],
	[9, 0, 1, 8, 4, 7, 2, 3, 11, 255, 255, 255, 255, 255, 255, 255],
	[4, 7, 11, 9, 4, 11, 9, 11, 2, 9, 2, 1, 255, 255, 255, 255],
	[3, 10, 1, 3, 11, 10, 7, 8, 4, 255, 255, 255, 255, 255, 255, 255],
	[1, 11, 10, 1, 4, 11, 1, 0, 4, 7, 11, 4, 255, 255, 255, 255],
	[4, 7, 8, 9, 0, 11, 9, 11, 10, 11, 0, 3, 255, 255, 255, 255],
	[4, 7, 11, 4, 11, 9, 9, 11, 10, 255, 255, 255, 255, 255, 255, 255],
	[9, 5, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[9, 5, 4, 0, 8, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 5, 4, 1, 5, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[8, 5, 4, 8, 3, 5, 3, 1, 5, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 10, 9, 5, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[3, 0, 8, 1, 2, 10, 4, 9, 5, 255, 255, 255, 255, 255, 255, 255],
	[5, 2, 10, 5, 4, 2, 4, 0, 2, 255, 255, 255, 255, 255, 255, 255],
	[2, 10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8, 255, 255, 255, 255],
	[9, 5, 4, 2, 3, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 11, 2, 0, 8, 11, 4, 9, 5, 255, 255, 255, 255, 255, 255, 255],
	[0, 5, 4, 0, 1, 5, 2, 3, 11, 255, 255, 255, 255, 255, 255, 255],
	[2, 1, 5, 2, 5, 8, 2, 8, 11, 4, 8, 5, 255, 255, 255, 255],
	[10, 3, 11, 10, 1, 3, 9, 5, 4, 255, 255, 255, 255, 255, 255, 255],
	[4, 9, 5, 0, 8, 1, 8, 10, 1, 8, 11, 10, 255, 255, 255, 255],
	[5, 4, 0, 5, 0, 11, 5, 11, 10, 11, 0, 3, 255, 255, 255, 255],
	[5, 4, 8, 5, 8, 10, 10, 8, 11, 255, 255, 255, 255, 255, 255, 255],
	[9, 7, 8, 5, 7, 9, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[9, 3, 0, 9, 5, 3, 5, 7, 3, 255, 255, 255, 255, 255, 255, 255],
	[0, 7, 8, 0, 1, 7, 1, 5, 7, 255, 255, 255, 255, 255, 255, 255],
	[1, 5, 3, 3, 5, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[9, 7, 8, 9, 5, 7, 10, 1, 2, 255, 255, 255, 255, 255, 255, 255],
	[10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3, 255, 255, 255, 255],
	[8, 0, 2, 8, 2, 5, 8, 5, 7, 10, 5, 2, 255, 255, 255, 255],
	[2, 10, 5, 2, 5, 3, 3, 5, 7, 255, 255, 255, 255, 255, 255, 255],
	[7, 9, 5, 7, 8, 9, 3, 11, 2, 255, 255, 255, 255, 255, 255, 255],
	[9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7, 11, 255, 255, 255, 255],
	[2, 3, 11, 0, 1, 8, 1, 7, 8, 1, 5, 7, 255, 255, 255, 255],
	[11, 2, 1, 11, 1, 7, 7, 1, 5, 255, 255, 255, 255, 255, 255, 255],
	[9, 5, 8, 8, 5, 7, 10, 1, 3, 10, 3, 11, 255, 255, 255, 255],
	[5, 7, 0, 5, 0, 9, 7, 11, 0, 1, 0, 10, 11, 10, 0, 255],
	[11, 10, 0, 11, 0, 3, 10, 5, 0, 8, 0, 7, 5, 7, 0, 255],
	[11, 10, 5, 7, 11, 5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[10, 6, 5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 3, 5, 10, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[9, 0, 1, 5, 10, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 8, 3, 1, 9, 8, 5, 10, 6, 255, 255, 255, 255, 255, 255, 255],
	[1, 6, 5, 2, 6, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 6, 5, 1, 2, 6, 3, 0, 8, 255, 255, 255, 255, 255, 255, 255],
	[9, 6, 5, 9, 0, 6, 0, 2, 6, 255, 255, 255, 255, 255, 255, 255],
	[5, 9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8, 255, 255, 255, 255],
	[2, 3, 11, 10, 6, 5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[11, 0, 8, 11, 2, 0, 10, 6, 5, 255, 255, 255, 255, 255, 255, 255],
	[0, 1, 9, 2, 3, 11, 5, 10, 6, 255, 255, 255, 255, 255, 255, 255],
	[5, 10, 6, 1, 9, 2, 9, 11, 2, 9, 8, 11, 255, 255, 255, 255],
	[6, 3, 11, 6, 5, 3, 5, 1, 3, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 11, 0, 11, 5, 0, 5, 1, 5, 11, 6, 255, 255, 255, 255],
	[3, 11, 6, 0, 3, 6, 0, 6, 5, 0, 5, 9, 255, 255, 255, 255],
	[6, 5, 9, 6, 9, 11, 11, 9, 8, 255, 255, 255, 255, 255, 255, 255],
	[5, 10, 6, 4, 7, 8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 3, 0, 4, 7, 3, 6, 5, 10, 255, 255, 255, 255, 255, 255, 255],
	[1, 9, 0, 5, 10, 6, 8, 4, 7, 255, 255, 255, 255, 255, 255, 255],
	[10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4, 255, 255, 255, 255],
	[6, 1, 2, 6, 5, 1, 4, 7, 8, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4, 7, 255, 255, 255, 255],
	[8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6, 255, 255, 255, 255],
	[7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9, 6, 2, 6, 9, 255],
	[3, 11, 2, 7, 8, 4, 10, 6, 5, 255, 255, 255, 255, 255, 255, 255],
	[5, 10, 6, 4, 7, 2, 4, 2, 0, 2, 7, 11, 255, 255, 255, 255],
	[0, 1, 9, 4, 7, 8, 2, 3, 11, 5, 10, 6, 255, 255, 255, 255],
	[9, 2, 1, 9, 11, 2, 9, 4, 11, 7, 11, 4, 5, 10, 6, 255],
	[8, 4, 7, 3, 11, 5, 3, 5, 1, 5, 11, 6, 255, 255, 255, 255],
	[5, 1, 11, 5, 11, 6, 1, 0, 11, 7, 11, 4, 0, 4, 11, 255],
	[0, 5, 9, 0, 6, 5, 0, 3, 6, 11, 6, 3, 8, 4, 7, 255],
	[6, 5, 9, 6, 9, 11, 4, 7, 9, 7, 11, 9, 255, 255, 255, 255],
	[10, 4, 9, 6, 4, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 10, 6, 4, 9, 10, 0, 8, 3, 255, 255, 255, 255, 255, 255, 255],
	[10, 0, 1, 10, 6, 0, 6, 4, 0, 255, 255, 255, 255, 255, 255, 255],
	[8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1, 10, 255, 255, 255, 255],
	[1, 4, 9, 1, 2, 4, 2, 6, 4, 255, 255, 255, 255, 255, 255, 255],
	[3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4, 255, 255, 255, 255],
	[0, 2, 4, 4, 2, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[8, 3, 2, 8, 2, 4, 4, 2, 6, 255, 255, 255, 255, 255, 255, 255],
	[10, 4, 9, 10, 6, 4, 11, 2, 3, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 2, 2, 8, 11, 4, 9, 10, 4, 10, 6, 255, 255, 255, 255],
	[3, 11, 2, 0, 1, 6, 0, 6, 4, 6, 1, 10, 255, 255, 255, 255],
	[6, 4, 1, 6, 1, 10, 4, 8, 1, 2, 1, 11, 8, 11, 1, 255],
	[9, 6, 4, 9, 3, 6, 9, 1, 3, 11, 6, 3, 255, 255, 255, 255],
	[8, 11, 1, 8, 1, 0, 11, 6, 1, 9, 1, 4, 6, 4, 1, 255],
	[3, 11, 6, 3, 6, 0, 0, 6, 4, 255, 255, 255, 255, 255, 255, 255],
	[6, 4, 8, 11, 6, 8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[7, 10, 6, 7, 8, 10, 8, 9, 10, 255, 255, 255, 255, 255, 255, 255],
	[0, 7, 3, 0, 10, 7, 0, 9, 10, 6, 7, 10, 255, 255, 255, 255],
	[10, 6, 7, 1, 10, 7, 1, 7, 8, 1, 8, 0, 255, 255, 255, 255],
	[10, 6, 7, 10, 7, 1, 1, 7, 3, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7, 255, 255, 255, 255],
	[2, 6, 9, 2, 9, 1, 6, 7, 9, 0, 9, 3, 7, 3, 9, 255],
	[7, 8, 0, 7, 0, 6, 6, 0, 2, 255, 255, 255, 255, 255, 255, 255],
	[7, 3, 2, 6, 7, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[2, 3, 11, 10, 6, 8, 10, 8, 9, 8, 6, 7, 255, 255, 255, 255],
	[2, 0, 7, 2, 7, 11, 0, 9, 7, 6, 7, 10, 9, 10, 7, 255],
	[1, 8, 0, 1, 7, 8, 1, 10, 7, 6, 7, 10, 2, 3, 11, 255],
	[11, 2, 1, 11, 1, 7, 10, 6, 1, 6, 7, 1, 255, 255, 255, 255],
	[8, 9, 6, 8, 6, 7, 9, 1, 6, 11, 6, 3, 1, 3, 6, 255],
	[0, 9, 1, 11, 6, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[7, 8, 0, 7, 0, 6, 3, 11, 0, 11, 6, 0, 255, 255, 255, 255],
	[7, 11, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[7, 6, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[3, 0, 8, 11, 7, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 1, 9, 11, 7, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[8, 1, 9, 8, 3, 1, 11, 7, 6, 255, 255, 255, 255, 255, 255, 255],
	[10, 1, 2, 6, 11, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 10, 3, 0, 8, 6, 11, 7, 255, 255, 255, 255, 255, 255, 255],
	[2, 9, 0, 2, 10, 9, 6, 11, 7, 255, 255, 255, 255, 255, 255, 255],
	[6, 11, 7, 2, 10, 3, 10, 8, 3, 10, 9, 8, 255, 255, 255, 255],
	[7, 2, 3, 6, 2, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[7, 0, 8, 7, 6, 0, 6, 2, 0, 255, 255, 255, 255, 255, 255, 255],
	[2, 7, 6, 2, 3, 7, 0, 1, 9, 255, 255, 255, 255, 255, 255, 255],
	[1, 6, 2, 1, 8, 6, 1, 9, 8, 8, 7, 6, 255, 255, 255, 255],
	[10, 7, 6, 10, 1, 7, 1, 3, 7, 255, 255, 255, 255, 255, 255, 255],
	[10, 7, 6, 1, 7, 10, 1, 8, 7, 1, 0, 8, 255, 255, 255, 255],
	[0, 3, 7, 0, 7, 10, 0, 10, 9, 6, 10, 7, 255, 255, 255, 255],
	[7, 6, 10, 7, 10, 8, 8, 10, 9, 255, 255, 255, 255, 255, 255, 255],
	[6, 8, 4, 11, 8, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[3, 6, 11, 3, 0, 6, 0, 4, 6, 255, 255, 255, 255, 255, 255, 255],
	[8, 6, 11, 8, 4, 6, 9, 0, 1, 255, 255, 255, 255, 255, 255, 255],
	[9, 4, 6, 9, 6, 3, 9, 3, 1, 11, 3, 6, 255, 255, 255, 255],
	[6, 8, 4, 6, 11, 8, 2, 10, 1, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 10, 3, 0, 11, 0, 6, 11, 0, 4, 6, 255, 255, 255, 255],
	[4, 11, 8, 4, 6, 11, 0, 2, 9, 2, 10, 9, 255, 255, 255, 255],
	[10, 9, 3, 10, 3, 2, 9, 4, 3, 11, 3, 6, 4, 6, 3, 255],
	[8, 2, 3, 8, 4, 2, 4, 6, 2, 255, 255, 255, 255, 255, 255, 255],
	[0, 4, 2, 4, 6, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 9, 0, 2, 3, 4, 2, 4, 6, 4, 3, 8, 255, 255, 255, 255],
	[1, 9, 4, 1, 4, 2, 2, 4, 6, 255, 255, 255, 255, 255, 255, 255],
	[8, 1, 3, 8, 6, 1, 8, 4, 6, 6, 10, 1, 255, 255, 255, 255],
	[10, 1, 0, 10, 0, 6, 6, 0, 4, 255, 255, 255, 255, 255, 255, 255],
	[4, 6, 3, 4, 3, 8, 6, 10, 3, 0, 3, 9, 10, 9, 3, 255],
	[10, 9, 4, 6, 10, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 9, 5, 7, 6, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 3, 4, 9, 5, 11, 7, 6, 255, 255, 255, 255, 255, 255, 255],
	[5, 0, 1, 5, 4, 0, 7, 6, 11, 255, 255, 255, 255, 255, 255, 255],
	[11, 7, 6, 8, 3, 4, 3, 5, 4, 3, 1, 5, 255, 255, 255, 255],
	[9, 5, 4, 10, 1, 2, 7, 6, 11, 255, 255, 255, 255, 255, 255, 255],
	[6, 11, 7, 1, 2, 10, 0, 8, 3, 4, 9, 5, 255, 255, 255, 255],
	[7, 6, 11, 5, 4, 10, 4, 2, 10, 4, 0, 2, 255, 255, 255, 255],
	[3, 4, 8, 3, 5, 4, 3, 2, 5, 10, 5, 2, 11, 7, 6, 255],
	[7, 2, 3, 7, 6, 2, 5, 4, 9, 255, 255, 255, 255, 255, 255, 255],
	[9, 5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7, 255, 255, 255, 255],
	[3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0, 255, 255, 255, 255],
	[6, 2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8, 255],
	[9, 5, 4, 10, 1, 6, 1, 7, 6, 1, 3, 7, 255, 255, 255, 255],
	[1, 6, 10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4, 255],
	[4, 0, 10, 4, 10, 5, 0, 3, 10, 6, 10, 7, 3, 7, 10, 255],
	[7, 6, 10, 7, 10, 8, 5, 4, 10, 4, 8, 10, 255, 255, 255, 255],
	[6, 9, 5, 6, 11, 9, 11, 8, 9, 255, 255, 255, 255, 255, 255, 255],
	[3, 6, 11, 0, 6, 3, 0, 5, 6, 0, 9, 5, 255, 255, 255, 255],
	[0, 11, 8, 0, 5, 11, 0, 1, 5, 5, 6, 11, 255, 255, 255, 255],
	[6, 11, 3, 6, 3, 5, 5, 3, 1, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 10, 9, 5, 11, 9, 11, 8, 11, 5, 6, 255, 255, 255, 255],
	[0, 11, 3, 0, 6, 11, 0, 9, 6, 5, 6, 9, 1, 2, 10, 255],
	[11, 8, 5, 11, 5, 6, 8, 0, 5, 10, 5, 2, 0, 2, 5, 255],
	[6, 11, 3, 6, 3, 5, 2, 10, 3, 10, 5, 3, 255, 255, 255, 255],
	[5, 8, 9, 5, 2, 8, 5, 6, 2, 3, 8, 2, 255, 255, 255, 255],
	[9, 5, 6, 9, 6, 0, 0, 6, 2, 255, 255, 255, 255, 255, 255, 255],
	[1, 5, 8, 1, 8, 0, 5, 6, 8, 3, 8, 2, 6, 2, 8, 255],
	[1, 5, 6, 2, 1, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 3, 6, 1, 6, 10, 3, 8, 6, 5, 6, 9, 8, 9, 6, 255],
	[10, 1, 0, 10, 0, 6, 9, 5, 0, 5, 6, 0, 255, 255, 255, 255],
	[0, 3, 8, 5, 6, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[10, 5, 6, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[11, 5, 10, 7, 5, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[11, 5, 10, 11, 7, 5, 8, 3, 0, 255, 255, 255, 255, 255, 255, 255],
	[5, 11, 7, 5, 10, 11, 1, 9, 0, 255, 255, 255, 255, 255, 255, 255],
	[10, 7, 5, 10, 11, 7, 9, 8, 1, 8, 3, 1, 255, 255, 255, 255],
	[11, 1, 2, 11, 7, 1, 7, 5, 1, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 3, 1, 2, 7, 1, 7, 5, 7, 2, 11, 255, 255, 255, 255],
	[9, 7, 5, 9, 2, 7, 9, 0, 2, 2, 11, 7, 255, 255, 255, 255],
	[7, 5, 2, 7, 2, 11, 5, 9, 2, 3, 2, 8, 9, 8, 2, 255],
	[2, 5, 10, 2, 3, 5, 3, 7, 5, 255, 255, 255, 255, 255, 255, 255],
	[8, 2, 0, 8, 5, 2, 8, 7, 5, 10, 2, 5, 255, 255, 255, 255],
	[9, 0, 1, 5, 10, 3, 5, 3, 7, 3, 10, 2, 255, 255, 255, 255],
	[9, 8, 2, 9, 2, 1, 8, 7, 2, 10, 2, 5, 7, 5, 2, 255],
	[1, 3, 5, 3, 7, 5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 7, 0, 7, 1, 1, 7, 5, 255, 255, 255, 255, 255, 255, 255],
	[9, 0, 3, 9, 3, 5, 5, 3, 7, 255, 255, 255, 255, 255, 255, 255],
	[9, 8, 7, 5, 9, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[5, 8, 4, 5, 10, 8, 10, 11, 8, 255, 255, 255, 255, 255, 255, 255],
	[5, 0, 4, 5, 11, 0, 5, 10, 11, 11, 3, 0, 255, 255, 255, 255],
	[0, 1, 9, 8, 4, 10, 8, 10, 11, 10, 4, 5, 255, 255, 255, 255],
	[10, 11, 4, 10, 4, 5, 11, 3, 4, 9, 4, 1, 3, 1, 4, 255],
	[2, 5, 1, 2, 8, 5, 2, 11, 8, 4, 5, 8, 255, 255, 255, 255],
	[0, 4, 11, 0, 11, 3, 4, 5, 11, 2, 11, 1, 5, 1, 11, 255],
	[0, 2, 5, 0, 5, 9, 2, 11, 5, 4, 5, 8, 11, 8, 5, 255],
	[9, 4, 5, 2, 11, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[2, 5, 10, 3, 5, 2, 3, 4, 5, 3, 8, 4, 255, 255, 255, 255],
	[5, 10, 2, 5, 2, 4, 4, 2, 0, 255, 255, 255, 255, 255, 255, 255],
	[3, 10, 2, 3, 5, 10, 3, 8, 5, 4, 5, 8, 0, 1, 9, 255],
	[5, 10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2, 255, 255, 255, 255],
	[8, 4, 5, 8, 5, 3, 3, 5, 1, 255, 255, 255, 255, 255, 255, 255],
	[0, 4, 5, 1, 0, 5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5, 255, 255, 255, 255],
	[9, 4, 5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 11, 7, 4, 9, 11, 9, 10, 11, 255, 255, 255, 255, 255, 255, 255],
	[0, 8, 3, 4, 9, 7, 9, 11, 7, 9, 10, 11, 255, 255, 255, 255],
	[1, 10, 11, 1, 11, 4, 1, 4, 0, 7, 4, 11, 255, 255, 255, 255],
	[3, 1, 4, 3, 4, 8, 1, 10, 4, 7, 4, 11, 10, 11, 4, 255],
	[4, 11, 7, 9, 11, 4, 9, 2, 11, 9, 1, 2, 255, 255, 255, 255],
	[9, 7, 4, 9, 11, 7, 9, 1, 11, 2, 11, 1, 0, 8, 3, 255],
	[11, 7, 4, 11, 4, 2, 2, 4, 0, 255, 255, 255, 255, 255, 255, 255],
	[11, 7, 4, 11, 4, 2, 8, 3, 4, 3, 2, 4, 255, 255, 255, 255],
	[2, 9, 10, 2, 7, 9, 2, 3, 7, 7, 4, 9, 255, 255, 255, 255],
	[9, 10, 7, 9, 7, 4, 10, 2, 7, 8, 7, 0, 2, 0, 7, 255],
	[3, 7, 10, 3, 10, 2, 7, 4, 10, 1, 10, 0, 4, 0, 10, 255],
	[1, 10, 2, 8, 7, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 9, 1, 4, 1, 7, 7, 1, 3, 255, 255, 255, 255, 255, 255, 255],
	[4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1, 255, 255, 255, 255],
	[4, 0, 3, 7, 4, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[4, 8, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[9, 10, 8, 10, 11, 8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[3, 0, 9, 3, 9, 11, 11, 9, 10, 255, 255, 255, 255, 255, 255, 255],
	[0, 1, 10, 0, 10, 8, 8, 10, 11, 255, 255, 255, 255, 255, 255, 255],
	[3, 1, 10, 11, 3, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 2, 11, 1, 11, 9, 9, 11, 8, 255, 255, 255, 255, 255, 255, 255],
	[3, 0, 9, 3, 9, 11, 1, 2, 9, 2, 11, 9, 255, 255, 255, 255],
	[0, 2, 11, 8, 0, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[3, 2, 11, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[2, 3, 8, 2, 8, 10, 10, 8, 9, 255, 255, 255, 255, 255, 255, 255],
	[9, 10, 2, 0, 9, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[2, 3, 8, 2, 8, 10, 0, 1, 8, 1, 10, 8, 255, 255, 255, 255],
	[1, 10, 2, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[1, 3, 8, 9, 1, 8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 9, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[0, 3, 8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
	[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
];
