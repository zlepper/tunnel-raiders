use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::prelude::*;


pub struct WallGenerationArgs {
    pub has_north_wall: bool,
    pub has_south_wall: bool,
    pub has_east_wall: bool,
    pub has_west_wall: bool,
    pub has_north_west_wall: bool,
    pub has_north_east_wall: bool,
    pub has_south_west_wall: bool,
    pub has_south_east_wall: bool,
    pub width: f32,
    pub height: f32,
}

pub fn generate_wall_mesh(arg: WallGenerationArgs) -> Mesh {

    let top = arg.height / 2.0;
    let bottom = -top;

    let half_width = arg.width / 2.0;
    let middle = 0.0;


    let min_z = if arg.has_south_wall { -half_width } else { middle };
    let max_z = if arg.has_north_wall { half_width } else { middle };
    let min_x = if arg.has_west_wall { -half_width } else { middle };
    let max_x = if arg.has_east_wall { half_width } else { middle };

    let top_south_west = [min_x, top, min_z];
    let top_south_east = [max_x, top, min_z];
    let top_north_west = [min_x, top, max_z];
    let top_north_east = [max_x, top, max_z];

    let bottom_south_west = [-half_width, bottom, -half_width];
    let bottom_south_east = [-half_width, bottom, half_width];
    let bottom_north_west = [half_width, bottom, -half_width];
    let bottom_north_east = [half_width, bottom, half_width];

    // suppose Y-up right hand, and camera look from +z to -z
    let vertices = &[
        // Front
        (bottom_south_east, [0., 0., 1.0], [0., 0.]),
        (bottom_north_east, [0., 0., 1.0], [1.0, 0.]),
        (top_north_east, [0., 0., 1.0], [1.0, 1.0]),
        (top_north_west, [0., 0., 1.0], [0., 1.0]),
        // Back
        (top_south_west, [0., 0., -1.0], [1.0, 0.]),
        (top_south_east, [0., 0., -1.0], [0., 0.]),
        (bottom_north_west, [0., 0., -1.0], [0., 1.0]),
        (bottom_south_west, [0., 0., -1.0], [1.0, 1.0]),
        // Right
        (bottom_north_west, [1.0, 0., 0.], [0., 0.]),
        (top_south_east, [1.0, 0., 0.], [1.0, 0.]),
        (top_north_east, [1.0, 0., 0.], [1.0, 1.0]),
        (bottom_north_east, [1.0, 0., 0.], [0., 1.0]),
        // Left
        (bottom_south_east, [-1.0, 0., 0.], [1.0, 0.]),
        (top_north_west, [-1.0, 0., 0.], [0., 0.]),
        (top_south_west, [-1.0, 0., 0.], [0., 1.0]),
        (bottom_south_west, [-1.0, 0., 0.], [1.0, 1.0]),
        // Top
        (top_south_east, [0., 1.0, 0.], [1.0, 0.]),
        (top_south_west, [0., 1.0, 0.], [0., 0.]),
        (top_north_west, [0., 1.0, 0.], [0., 1.0]),
        (top_north_east, [0., 1.0, 0.], [1.0, 1.0]),
        // Bottom
        (bottom_north_east, [0., -1.0, 0.], [0., 0.]),
        (bottom_south_east, [0., -1.0, 0.], [1.0, 0.]),
        (bottom_south_west, [0., -1.0, 0.], [1.0, 1.0]),
        (bottom_north_west, [0., -1.0, 0.], [0., 1.0]),
    ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let indices = Indices::U32(vec![
        0, 1, 2, 2, 3, 0, // front
        4, 5, 6, 6, 7, 4, // back
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}
