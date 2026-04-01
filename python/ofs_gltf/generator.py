"""Generate glTF 2.0 / GLB files from kozijn data.

Generates a simple 3D representation with:
- Frame members as extruded rectangles
- Glass panels as thin transparent slabs
- Dividers as extruded rectangles
"""

import struct
import json
import base64
import math


def generate_gltf(kozijn_data: dict, output_path: str):
    """Generate a GLB (binary glTF) file from kozijn data."""
    frame = kozijn_data.get("frame", {})
    grid = kozijn_data.get("grid", {})
    cells = kozijn_data.get("cells", [])

    ow = frame.get("outerWidth", 1200)
    oh = frame.get("outerHeight", 1500)
    fw = frame.get("frameWidth", 67)
    fd = frame.get("frameDepth", 114)

    columns = grid.get("columns", [])
    rows = grid.get("rows", [])

    # Convert mm to meters for glTF (standard unit)
    scale = 0.001

    meshes = []
    nodes = []

    # Material indices: 0 = frame (wood), 1 = glass
    materials = [
        {
            "name": "Frame",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.55, 0.35, 0.2, 1.0],
                "metallicFactor": 0.0,
                "roughnessFactor": 0.8,
            }
        },
        {
            "name": "Glass",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.7, 0.85, 0.95, 0.3],
                "metallicFactor": 0.0,
                "roughnessFactor": 0.1,
            },
            "alphaMode": "BLEND",
        },
    ]

    all_positions = []
    all_indices = []
    mesh_primitives = []

    def add_box(x, y, z, w, h, d, material_idx):
        """Add a box mesh at position (x, y, z) with dimensions (w, h, d)."""
        base_idx = len(all_positions) // 3

        # 8 vertices of a box
        verts = [
            x, y, z,
            x + w, y, z,
            x + w, y + h, z,
            x, y + h, z,
            x, y, z + d,
            x + w, y, z + d,
            x + w, y + h, z + d,
            x, y + h, z + d,
        ]
        all_positions.extend(verts)

        # 12 triangles (6 faces x 2 triangles)
        faces = [
            0, 1, 2, 0, 2, 3,  # front
            4, 6, 5, 4, 7, 6,  # back
            0, 4, 5, 0, 5, 1,  # bottom
            2, 6, 7, 2, 7, 3,  # top
            0, 3, 7, 0, 7, 4,  # left
            1, 5, 6, 1, 6, 2,  # right
        ]
        indices = [f + base_idx for f in faces]
        all_indices.extend(indices)

        return {"material": material_idx, "start": len(all_indices) - 36, "count": 36}

    # Frame members (in meters)
    x0 = 0
    y0 = 0
    z0 = 0

    # Left stile
    add_box(x0 * scale, y0 * scale, z0 * scale,
            fw * scale, oh * scale, fd * scale, 0)
    # Right stile
    add_box((ow - fw) * scale, y0 * scale, z0 * scale,
            fw * scale, oh * scale, fd * scale, 0)
    # Top rail
    add_box(fw * scale, (oh - fw) * scale, z0 * scale,
            (ow - 2 * fw) * scale, fw * scale, fd * scale, 0)
    # Bottom rail
    add_box(fw * scale, y0 * scale, z0 * scale,
            (ow - 2 * fw) * scale, fw * scale, fd * scale, 0)

    # Vertical dividers
    x_acc = fw
    for i, col in enumerate(columns):
        col_size = col.get("size", 0)
        if col.get("dividerProfile"):
            add_box(x_acc * scale, fw * scale, z0 * scale,
                    fw * scale, (oh - 2 * fw) * scale, fd * scale, 0)
            x_acc += fw
        x_acc += col_size

    # Horizontal dividers
    y_acc = fw
    for i, row in enumerate(rows):
        row_size = row.get("size", 0)
        if row.get("dividerProfile"):
            add_box(fw * scale, y_acc * scale, z0 * scale,
                    (ow - 2 * fw) * scale, fw * scale, fd * scale, 0)
            y_acc += fw
        y_acc += row_size

    # Glass panels per cell
    num_cols = len(columns)
    glass_depth = 4 * scale  # 4mm glass thickness
    glass_z = (fd / 2 - 2) * scale  # centered in frame

    y_acc = fw
    for row_idx, row in enumerate(rows):
        x_acc = fw
        for col_idx, col in enumerate(columns):
            cell_idx = row_idx * num_cols + col_idx
            cell = cells[cell_idx] if cell_idx < len(cells) else {}
            panel_type = cell.get("panelType", "fixed_glass")

            if panel_type != "ventilation":
                # Glass/panel slab
                clearance = 4  # mm
                gw = (col.get("size", 0) - 2 * clearance) * scale
                gh = (row.get("size", 0) - 2 * clearance) * scale
                gx = (x_acc + clearance) * scale
                gy = (y_acc + clearance) * scale

                mat_idx = 1 if panel_type != "panel" else 0
                add_box(gx, gy, glass_z, gw, gh, glass_depth, mat_idx)

            if col.get("dividerProfile"):
                x_acc += fw
            x_acc += col.get("size", 0)

        if row.get("dividerProfile"):
            y_acc += fw
        y_acc += row.get("size", 0)

    # Build GLB binary
    _write_glb(output_path, all_positions, all_indices, materials)


def _write_glb(output_path, positions, indices, materials):
    """Write a complete GLB file."""
    # Pack binary buffer
    pos_bytes = struct.pack(f"<{len(positions)}f", *positions)
    idx_bytes = struct.pack(f"<{len(indices)}I", *indices)

    pos_byte_length = len(pos_bytes)
    idx_byte_length = len(idx_bytes)

    # Pad to 4-byte alignment
    pos_padding = (4 - pos_byte_length % 4) % 4
    idx_padding = (4 - idx_byte_length % 4) % 4

    buffer_data = pos_bytes + b'\x00' * pos_padding + idx_bytes + b'\x00' * idx_padding
    total_buffer_length = len(buffer_data)

    idx_offset = pos_byte_length + pos_padding

    # Calculate bounds for positions
    num_verts = len(positions) // 3
    min_pos = [float('inf')] * 3
    max_pos = [float('-inf')] * 3
    for i in range(num_verts):
        for j in range(3):
            v = positions[i * 3 + j]
            min_pos[j] = min(min_pos[j], v)
            max_pos[j] = max(max_pos[j], v)

    # Build glTF JSON
    gltf = {
        "asset": {"version": "2.0", "generator": "Open Frame Studio"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": "Kozijn"}],
        "meshes": [{
            "primitives": [{
                "attributes": {"POSITION": 0},
                "indices": 1,
                "material": 0,
            }]
        }],
        "materials": materials,
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,  # FLOAT
                "count": num_verts,
                "type": "VEC3",
                "min": min_pos,
                "max": max_pos,
            },
            {
                "bufferView": 1,
                "componentType": 5125,  # UNSIGNED_INT
                "count": len(indices),
                "type": "SCALAR",
                "min": [0],
                "max": [num_verts - 1] if num_verts > 0 else [0],
            },
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": pos_byte_length,
                "target": 34962,  # ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": idx_offset,
                "byteLength": idx_byte_length,
                "target": 34963,  # ELEMENT_ARRAY_BUFFER
            },
        ],
        "buffers": [{
            "byteLength": total_buffer_length,
        }],
    }

    # Encode JSON chunk
    json_str = json.dumps(gltf, separators=(',', ':'))
    json_bytes = json_str.encode('utf-8')
    json_padding = (4 - len(json_bytes) % 4) % 4
    json_bytes_padded = json_bytes + b' ' * json_padding

    # Write GLB
    with open(output_path, 'wb') as f:
        # Header
        f.write(struct.pack('<I', 0x46546C67))  # magic: glTF
        f.write(struct.pack('<I', 2))             # version
        total_length = 12 + 8 + len(json_bytes_padded) + 8 + total_buffer_length
        f.write(struct.pack('<I', total_length))

        # JSON chunk
        f.write(struct.pack('<I', len(json_bytes_padded)))
        f.write(struct.pack('<I', 0x4E4F534A))  # JSON
        f.write(json_bytes_padded)

        # BIN chunk
        f.write(struct.pack('<I', total_buffer_length))
        f.write(struct.pack('<I', 0x004E4942))  # BIN
        f.write(buffer_data)
