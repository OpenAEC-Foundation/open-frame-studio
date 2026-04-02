//! GLB (binary glTF 2.0) export for kozijnen.
//!
//! Generates a simple 3D representation with frame members as extruded
//! rectangles, glass panels as thin transparent slabs, and dividers.

use std::io::Write;

use crate::kozijn::{Kozijn, PanelType};

/// Generate a GLB file from a kozijn definition.
pub fn generate_glb(kozijn: &Kozijn, output_path: &str) -> Result<(), String> {
    let frame = &kozijn.frame;
    let grid = &kozijn.grid;
    let cells = &kozijn.cells;

    let ow = frame.outer_width;
    let oh = frame.outer_height;
    let fw = frame.frame_width;
    let fd = frame.frame_depth;

    let scale: f64 = 0.001; // mm to meters

    let mut positions: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut add_box = |x: f64, y: f64, z: f64, w: f64, h: f64, d: f64| {
        let base_idx = (positions.len() / 3) as u32;

        // 8 vertices
        let verts: [(f64, f64, f64); 8] = [
            (x, y, z),
            (x + w, y, z),
            (x + w, y + h, z),
            (x, y + h, z),
            (x, y, z + d),
            (x + w, y, z + d),
            (x + w, y + h, z + d),
            (x, y + h, z + d),
        ];
        for (vx, vy, vz) in &verts {
            positions.push(*vx as f32);
            positions.push(*vy as f32);
            positions.push(*vz as f32);
        }

        // 12 triangles (6 faces x 2)
        let faces: [u32; 36] = [
            0, 1, 2, 0, 2, 3, // front
            4, 6, 5, 4, 7, 6, // back
            0, 4, 5, 0, 5, 1, // bottom
            2, 6, 7, 2, 7, 3, // top
            0, 3, 7, 0, 7, 4, // left
            1, 5, 6, 1, 6, 2, // right
        ];
        for f in &faces {
            indices.push(f + base_idx);
        }
    };

    // Frame members
    // Left stile
    add_box(0.0, 0.0, 0.0, fw * scale, oh * scale, fd * scale);
    // Right stile
    add_box((ow - fw) * scale, 0.0, 0.0, fw * scale, oh * scale, fd * scale);
    // Top rail
    add_box(
        fw * scale,
        (oh - fw) * scale,
        0.0,
        (ow - 2.0 * fw) * scale,
        fw * scale,
        fd * scale,
    );
    // Bottom rail
    add_box(
        fw * scale,
        0.0,
        0.0,
        (ow - 2.0 * fw) * scale,
        fw * scale,
        fd * scale,
    );

    // Vertical dividers
    let mut x_acc = fw;
    for col in &grid.columns {
        if col.divider_profile.is_some() {
            add_box(
                x_acc * scale,
                fw * scale,
                0.0,
                fw * scale,
                (oh - 2.0 * fw) * scale,
                fd * scale,
            );
            x_acc += fw;
        }
        x_acc += col.size;
    }

    // Horizontal dividers
    let mut y_acc = fw;
    for row in &grid.rows {
        if row.divider_profile.is_some() {
            add_box(
                fw * scale,
                y_acc * scale,
                0.0,
                (ow - 2.0 * fw) * scale,
                fw * scale,
                fd * scale,
            );
            y_acc += fw;
        }
        y_acc += row.size;
    }

    // Glass panels per cell
    let num_cols = grid.columns.len();
    let glass_depth = 4.0 * scale;
    let glass_z = (fd / 2.0 - 2.0) * scale;
    let clearance = 4.0; // mm

    let mut y_acc = fw;
    for (row_idx, row) in grid.rows.iter().enumerate() {
        let mut x_acc = fw;
        for (col_idx, col) in grid.columns.iter().enumerate() {
            let cell_idx = row_idx * num_cols + col_idx;
            let panel_type = cells
                .get(cell_idx)
                .map(|c| c.panel_type)
                .unwrap_or(PanelType::FixedGlass);

            if panel_type != PanelType::Ventilation {
                let gw = (col.size - 2.0 * clearance) * scale;
                let gh = (row.size - 2.0 * clearance) * scale;
                let gx = (x_acc + clearance) * scale;
                let gy = (y_acc + clearance) * scale;
                add_box(gx, gy, glass_z, gw, gh, glass_depth);
            }

            if col.divider_profile.is_some() {
                x_acc += fw;
            }
            x_acc += col.size;
        }
        if row.divider_profile.is_some() {
            y_acc += fw;
        }
        y_acc += row.size;
    }

    // Materials
    let materials = serde_json::json!([
        {
            "name": "Frame",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.55, 0.35, 0.2, 1.0],
                "metallicFactor": 0.0,
                "roughnessFactor": 0.8
            }
        },
        {
            "name": "Glass",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.7, 0.85, 0.95, 0.3],
                "metallicFactor": 0.0,
                "roughnessFactor": 0.1
            },
            "alphaMode": "BLEND"
        }
    ]);

    write_glb(output_path, &positions, &indices, &materials)
}

fn write_glb(
    output_path: &str,
    positions: &[f32],
    indices: &[u32],
    materials: &serde_json::Value,
) -> Result<(), String> {
    // Pack binary buffer
    let pos_bytes: Vec<u8> = positions
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let idx_bytes: Vec<u8> = indices
        .iter()
        .flat_map(|i| i.to_le_bytes())
        .collect();

    let pos_byte_length = pos_bytes.len();
    let idx_byte_length = idx_bytes.len();

    let pos_padding = (4 - pos_byte_length % 4) % 4;
    let idx_padding = (4 - idx_byte_length % 4) % 4;

    let mut buffer_data = Vec::new();
    buffer_data.extend_from_slice(&pos_bytes);
    buffer_data.extend(std::iter::repeat(0u8).take(pos_padding));
    buffer_data.extend_from_slice(&idx_bytes);
    buffer_data.extend(std::iter::repeat(0u8).take(idx_padding));

    let total_buffer_length = buffer_data.len();
    let idx_offset = pos_byte_length + pos_padding;

    // Calculate bounds
    let num_verts = positions.len() / 3;
    let mut min_pos = [f32::INFINITY; 3];
    let mut max_pos = [f32::NEG_INFINITY; 3];
    for i in 0..num_verts {
        for j in 0..3 {
            let v = positions[i * 3 + j];
            min_pos[j] = min_pos[j].min(v);
            max_pos[j] = max_pos[j].max(v);
        }
    }

    // Build glTF JSON
    let gltf = serde_json::json!({
        "asset": {"version": "2.0", "generator": "Open Frame Studio"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": "Kozijn"}],
        "meshes": [{
            "primitives": [{
                "attributes": {"POSITION": 0},
                "indices": 1,
                "material": 0
            }]
        }],
        "materials": materials,
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": num_verts,
                "type": "VEC3",
                "min": [min_pos[0], min_pos[1], min_pos[2]],
                "max": [max_pos[0], max_pos[1], max_pos[2]]
            },
            {
                "bufferView": 1,
                "componentType": 5125,
                "count": indices.len(),
                "type": "SCALAR",
                "min": [0],
                "max": [if num_verts > 0 { num_verts - 1 } else { 0 }]
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": pos_byte_length,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": idx_offset,
                "byteLength": idx_byte_length,
                "target": 34963
            }
        ],
        "buffers": [{
            "byteLength": total_buffer_length
        }]
    });

    let json_str = serde_json::to_string(&gltf).map_err(|e| e.to_string())?;
    let json_bytes = json_str.as_bytes();
    let json_padding = (4 - json_bytes.len() % 4) % 4;
    let json_padded_len = json_bytes.len() + json_padding;

    let total_length: u32 =
        (12 + 8 + json_padded_len + 8 + total_buffer_length) as u32;

    let mut file =
        std::fs::File::create(output_path).map_err(|e| format!("Kan bestand niet aanmaken: {}", e))?;

    // Header
    file.write_all(&0x46546C67u32.to_le_bytes()).map_err(|e| e.to_string())?; // magic: glTF
    file.write_all(&2u32.to_le_bytes()).map_err(|e| e.to_string())?; // version
    file.write_all(&total_length.to_le_bytes()).map_err(|e| e.to_string())?;

    // JSON chunk
    file.write_all(&(json_padded_len as u32).to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&0x4E4F534Au32.to_le_bytes())
        .map_err(|e| e.to_string())?; // JSON
    file.write_all(json_bytes).map_err(|e| e.to_string())?;
    for _ in 0..json_padding {
        file.write_all(b" ").map_err(|e| e.to_string())?;
    }

    // BIN chunk
    file.write_all(&(total_buffer_length as u32).to_le_bytes())
        .map_err(|e| e.to_string())?;
    file.write_all(&0x004E4942u32.to_le_bytes())
        .map_err(|e| e.to_string())?; // BIN
    file.write_all(&buffer_data).map_err(|e| e.to_string())?;

    Ok(())
}
