//! CadDocument setup and DXF serialization via acad-ts shell template + Rust ENTITIES.

use crate::sanitize::{inject_header_extents, sanitize_dxf_string};
use crate::template::{parse_shell_info, shell_template, write_dxf_from_template};

pub const PAD_IN: f64 = 1.15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CadExtentInches {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug, Clone)]
pub(crate) enum EntityKind {
    Line {
        layer: String,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    LwPolyline {
        layer: String,
        pts: Vec<(f64, f64)>,
        closed: bool,
    },
    Text {
        layer: String,
        x: f64,
        y: f64,
        height: f64,
        value: String,
        h_align: TextHAlign,
        v_align: TextVAlign,
    },
    Solid {
        layer: String,
        corners: [(f64, f64); 4],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextHAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVAlign {
    Baseline,
    Middle,
    Bottom,
    Top,
}

impl TextHAlign {
    fn dxf_code(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Center => 1,
            Self::Right => 2,
        }
    }
}

impl TextVAlign {
    fn dxf_code(self) -> i32 {
        match self {
            Self::Baseline => 0,
            Self::Bottom => 1,
            Self::Middle => 2,
            Self::Top => 3,
        }
    }
}

pub struct CadDocument {
    pub(crate) extent: CadExtentInches,
    pub(crate) entities: Vec<EntityKind>,
}

impl CadDocument {
    pub fn extent(&self) -> CadExtentInches {
        self.extent
    }
}

pub fn create_revit_cad_document(ext: CadExtentInches) -> CadDocument {
    CadDocument {
        extent: ext,
        entities: Vec::new(),
    }
}

pub fn serialize_revit_dxf(doc: &CadDocument) -> String {
    serialize_revit_dxf_with_filter(doc, &EntityFilter::default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityTypeFilter {
    Line,
    LwPolyline,
    Text,
    Solid,
}

#[derive(Debug, Clone, Default)]
pub struct EntityFilter {
    pub layers: Option<Vec<String>>,
    pub include_kinds: Option<Vec<EntityTypeFilter>>,
}

impl EntityFilter {
    pub fn layers(names: &[&str]) -> Self {
        Self {
            layers: Some(names.iter().map(|s| (*s).to_string()).collect()),
            include_kinds: None,
        }
    }

    pub fn kinds(kinds: &[EntityTypeFilter]) -> Self {
        Self {
            layers: None,
            include_kinds: Some(kinds.to_vec()),
        }
    }

    pub(crate) fn apply(&self, entities: &[EntityKind]) -> Vec<EntityKind> {
        entities
            .iter()
            .filter(|entity| self.matches(entity))
            .cloned()
            .collect()
    }

    fn matches(&self, entity: &EntityKind) -> bool {
        if let Some(layers) = &self.layers {
            if !layers.iter().any(|layer| layer == entity_layer(entity)) {
                return false;
            }
        }
        if let Some(kinds) = &self.include_kinds {
            if !kinds.iter().any(|kind| *kind == entity_type(entity)) {
                return false;
            }
        }
        true
    }
}

fn entity_layer(entity: &EntityKind) -> &str {
    match entity {
        EntityKind::Line { layer, .. }
        | EntityKind::LwPolyline { layer, .. }
        | EntityKind::Text { layer, .. }
        | EntityKind::Solid { layer, .. } => layer,
    }
}

fn entity_type(entity: &EntityKind) -> EntityTypeFilter {
    match entity {
        EntityKind::Line { .. } => EntityTypeFilter::Line,
        EntityKind::LwPolyline { .. } => EntityTypeFilter::LwPolyline,
        EntityKind::Text { .. } => EntityTypeFilter::Text,
        EntityKind::Solid { .. } => EntityTypeFilter::Solid,
    }
}

pub fn serialize_revit_dxf_with_filter(doc: &CadDocument, filter: &EntityFilter) -> String {
    let entities = filter.apply(&doc.entities);
    let filtered = CadDocument {
        extent: doc.extent,
        entities,
    };
    let raw = write_dxf(&filtered);
    let with_extents = inject_header_extents(&raw, filtered.extent);
    sanitize_dxf_string(&with_extents)
}

struct EntityWriter {
    out: String,
    next_handle: u32,
    model_owner: String,
}

impl EntityWriter {
    fn new(shell: &crate::template::ShellInfo) -> Self {
        Self {
            out: String::new(),
            next_handle: shell.next_entity_handle,
            model_owner: shell.model_space_owner.clone(),
        }
    }

    fn pair(&mut self, code: i32, value: impl std::fmt::Display) {
        use std::fmt::Write;
        let _ = write!(self.out, "  {code}\n{value}\n");
    }

    fn next_handle(&mut self) -> String {
        let handle = format!("{:X}", self.next_handle);
        self.next_handle += 1;
        handle
    }

    /// Match v6 acad-ts entity header: ByLayer color/linetype, hairline weight.
    fn entity_header(&mut self, kind: &str, layer: &str) {
        let handle = self.next_handle();
        let owner = self.model_owner.clone();
        self.pair(0, kind);
        self.pair(5, handle);
        self.pair(330, owner);
        self.pair(100, "AcDbEntity");
        self.pair(8, layer);
        self.pair(6, "ByLayer");
        self.pair(62, 256);
        self.pair(48, "1.0");
        self.pair(60, 0);
        self.pair(67, 0);
        self.pair(370, 9);
    }

    fn write_line(&mut self, layer: &str, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.entity_header("LINE", layer);
        self.pair(100, "AcDbLine");
        self.pair(10, fmt_num(x1));
        self.pair(20, fmt_num(y1));
        self.pair(30, "0.0");
        self.pair(11, fmt_num(x2));
        self.pair(21, fmt_num(y2));
        self.pair(31, "0.0");
    }

    fn write_lwpolyline(&mut self, layer: &str, pts: &[(f64, f64)], closed: bool) {
        self.entity_header("LWPOLYLINE", layer);
        self.pair(100, "AcDbPolyline");
        self.pair(90, pts.len());
        self.pair(70, if closed { 1 } else { 0 });
        self.pair(43, "0.0");
        self.pair(38, "0.0");
        self.pair(39, "0.0");
        self.pair(210, "0.0");
        self.pair(220, "0.0");
        self.pair(230, "1.0");
        for (x, y) in pts {
            self.pair(10, fmt_num(*x));
            self.pair(20, fmt_num(*y));
            self.pair(40, "0.0");
            self.pair(41, "0.0");
            self.pair(42, "0.0");
        }
    }

    fn write_text(
        &mut self,
        layer: &str,
        x: f64,
        y: f64,
        height: f64,
        value: &str,
        h_align: TextHAlign,
        v_align: TextVAlign,
    ) {
        self.entity_header("TEXT", layer);
        self.pair(100, "AcDbText");
        self.pair(39, "0.0");
        self.pair(10, fmt_num(x));
        self.pair(20, fmt_num(y));
        self.pair(30, "0.0");
        self.pair(40, fmt_num(height));
        self.pair(1, value);
        self.pair(50, "0.0");
        self.pair(41, "1.0");
        self.pair(51, "0.0");
        self.pair(7, "Standard");
        self.pair(71, 0);
        self.pair(72, h_align.dxf_code());
        self.pair(11, fmt_num(x));
        self.pair(21, fmt_num(y));
        self.pair(31, "0.0");
        self.pair(210, "0.0");
        self.pair(220, "0.0");
        self.pair(230, "1.0");
        self.pair(100, "AcDbText");
        self.pair(73, v_align.dxf_code());
    }

    fn write_solid(&mut self, layer: &str, corners: &[(f64, f64); 4]) {
        self.entity_header("SOLID", layer);
        self.pair(100, "AcDbTrace");
        for (i, (x, y)) in corners.iter().enumerate() {
            let x_code = 10 + i as i32;
            let y_code = 20 + i as i32;
            self.pair(x_code, fmt_num(*x));
            self.pair(y_code, fmt_num(*y));
            self.pair(30 + i as i32, "0.0");
        }
        self.pair(39, "0.0");
        self.pair(210, "0.0");
        self.pair(220, "0.0");
        self.pair(230, "1.0");
    }

    fn write_entities(&mut self, entities: &[EntityKind]) {
        // SOLID-first matches v6 Revit-safe ordering.
        for entity in entities {
            if matches!(entity, EntityKind::Solid { .. }) {
                self.write_one(entity);
            }
        }
        for entity in entities {
            if !matches!(entity, EntityKind::Solid { .. }) {
                self.write_one(entity);
            }
        }
    }

    fn write_one(&mut self, entity: &EntityKind) {
        match entity {
            EntityKind::Line {
                layer,
                x1,
                y1,
                x2,
                y2,
            } => self.write_line(layer, *x1, *y1, *x2, *y2),
            EntityKind::LwPolyline {
                layer,
                pts,
                closed,
            } => self.write_lwpolyline(layer, pts, *closed),
            EntityKind::Text {
                layer,
                x,
                y,
                height,
                value,
                h_align,
                v_align,
            } => self.write_text(layer, *x, *y, *height, value, *h_align, *v_align),
            EntityKind::Solid { layer, corners } => self.write_solid(layer, corners),
        }
    }

    fn finish(self) -> String {
        self.out
    }
}

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() {
        format!("{n:.1}")
    } else {
        format!("{n}")
    }
}

fn write_dxf(doc: &CadDocument) -> String {
    let shell = parse_shell_info(shell_template());
    let mut writer = EntityWriter::new(&shell);
    writer.write_entities(&doc.entities);
    write_dxf_from_template(&writer.finish())
}
