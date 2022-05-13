# marching-cubes

## structure: 
```
VoxelObject {
	volumes: Vec<Volume>
}

Volume {
	chunks: HashMap<Chunk>,
	meshes: HashMap<Mesh>,
	node: Ref<MeshInstance>,
	mesh: Ref<ArrayMesh>,
	surface_level: u8,
	material: Ref<Material>,
}

enum Chunk {
	Ready(ChunkData)
	Empty
}

Mesh {
	verts: PoolArray<Vector3>
}

ChunkData {
	voxels: [u8; 32^3]
}

```
