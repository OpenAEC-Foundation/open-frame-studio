use crate::curved_timber::ArchJointKind;
use crate::production::{z_band_text, ArchJointItem, ProductionData};

pub(super) const KINDS: [ArchJointKind; 2] = [ArchJointKind::Springing, ArchJointKind::RingSeam];

pub(super) fn present(data: &[ProductionData], kind: ArchJointKind) -> bool {
    data.iter().flat_map(|p| &p.arch_joints).any(|j| j.joint.kind == kind)
}

pub(super) fn table_name(kind: ArchJointKind) -> &'static str {
    match kind { ArchJointKind::Springing => "Boogvoeten", ArchJointKind::RingSeam => "Ringnaden" }
}

#[cfg(feature = "export")]
pub(super) fn cut_headers(kind: ArchJointKind) -> [&'static str; 2] {
    match kind {
        ArchJointKind::Springing => ["Boogvoet begin", "Boogvoet eind"],
        ArchJointKind::RingSeam => ["Ringnaad begin", "Ringnaad eind"],
    }
}

pub(super) fn headers(kind: ArchJointKind) -> [&'static str; 28] {
    let mut h = HEADERS;
    if kind == ArchJointKind::RingSeam { h[1] = "Ringnaad"; }
    h
}

const HEADERS: [&str; 28] = [
    "Kozijn", "Boogvoet", "Penstuk", "Kop penstuk", "Gatstuk", "Kop gatstuk",
    "Schouderhoek penstuk (°)", "Schouderhoek gatstuk (°)", "Gevraagde pen (mm)",
    "Schouder buiten X (mm)", "Schouder buiten Y (mm)", "Schouder dag X (mm)", "Schouder dag Y (mm)",
    "Aantal pennen / gaten", "Penlengte (mm)", "Gatdiepte (mm)", "Bodemspeling (mm)",
    "Overmaat voorbij langpunt (mm)", "Penbreedte haaks op as (mm)",
    "Hart gatmond X (mm)", "Hart gatmond Y (mm)", "Hart langs schouder vanaf buitenpunt (mm)",
    "Insteekrichting vanaf X+ (° met klok mee)", "Z-banden vanaf binnen (mm)",
    "Hart gatbodem X (mm)", "Hart gatbodem Y (mm)", "Passing", "Referentie XY",
];

pub(super) enum Field {
    Text(String),
    Number(f64, usize),
    Empty,
}

impl Field {
    pub(super) fn csv(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Number(v, precision) => format!("{v:.precision$}"),
            Self::Empty => String::new(),
        }
    }
}

pub(super) fn fields(mark: &str, row: &ArchJointItem) -> Vec<Field> {
    use Field::{Empty, Number, Text};
    let j = &row.joint;
    let [outer, day] = j.shoulder_face;
    let mm = |n| Number(n, 1);
    let deg = |n| Number(n, 2);
    let mut cells = vec![Text(mark.into()), Text(row.side_nl().into()), Text(row.piece_id.clone()),
        Text(row.end.label_nl().into()), Text(row.receiver_piece_id.clone()), Text(row.receiver_end.label_nl().into()),
        deg(j.source_angle_deg), deg(j.receiver_angle_deg), mm(j.requested_length_mm),
        mm(outer[0]), mm(outer[1]), mm(day[0]), mm(day[1])];
    if let Some(t) = &j.tenon {
        let [x, y] = t.shoulder_center;
        let [dx, dy] = t.direction;
        let [a, b] = t.tip;
        cells.extend([Number(t.z_from_inside_mm.len() as f64, 0), mm(t.tenon_length_mm), mm(t.mortise_depth_mm),
            mm(t.bottom_clearance_mm), mm(t.blank_extension_mm), mm((a[0] - b[0]).hypot(a[1] - b[1])),
            mm(x), mm(y), mm((x - outer[0]).hypot(y - outer[1])),
            deg(dy.atan2(dx).to_degrees().rem_euclid(360.0)), Text(z_band_text(&t.z_from_inside_mm)),
            mm((t.bottom[0][0] + t.bottom[1][0]) * 0.5), mm((t.bottom[0][1] + t.bottom[1][1]) * 0.5)]);
    } else {
        cells.resize_with(26, || Empty);
    }
    cells.push(Text(j.issue.map(|issue| issue.to_string()).unwrap_or_default()));
    cells.push(Text("Kozijn linksboven, X rechts en Y omlaag".into()));
    cells
}
