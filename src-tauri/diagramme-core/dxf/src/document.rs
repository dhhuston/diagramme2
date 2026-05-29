//! CadDocument setup, layers, Arial Narrow STYLE, and DXF serialization.

use crate::sanitize::{inject_header_extents, sanitize_dxf_string};

pub const PAD_IN: f64 = 1.15;
const ACADVER: &str = "AC1015";

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
    let raw = write_dxf(doc);
    let with_extents = inject_header_extents(&raw, doc.extent);
    sanitize_dxf_string(&with_extents)
}

struct HandleGen {
    next: u32,
}

impl HandleGen {
    fn new() -> Self {
        Self { next: 0x10 }
    }

    fn next(&mut self) -> String {
        let handle = format!("{:X}", self.next);
        self.next += 1;
        handle
    }
}

struct DxfWriter {
    out: String,
    handles: HandleGen,
    model_space_handle: String,
    paper_space_handle: String,
}

impl DxfWriter {
    fn new() -> Self {
        let mut handles = HandleGen::new();
        let model_space_handle = handles.next();
        let paper_space_handle = handles.next();
        Self {
            out: String::new(),
            handles,
            model_space_handle,
            paper_space_handle,
        }
    }

    fn pair(&mut self, code: i32, value: impl std::fmt::Display) {
        use std::fmt::Write;
        let _ = write!(self.out, "  {code}\n{value}\n");
    }

    fn section(&mut self, name: &str) {
        self.pair(0, "SECTION");
        self.pair(2, name);
    }

    fn endsec(&mut self) {
        self.pair(0, "ENDSEC");
    }

    fn next_handle(&mut self) -> String {
        self.handles.next()
    }

    fn table(&mut self, name: &str, max_entries: i32) {
        self.pair(0, "TABLE");
        self.pair(2, name);
        let handle = self.next_handle();
        self.pair(5, handle);
        self.pair(330, "0");
        self.pair(100, "AcDbSymbolTable");
        self.pair(70, max_entries);
    }

    fn endtab(&mut self) {
        self.pair(0, "ENDTAB");
    }

    fn write_header(&mut self, ext: CadExtentInches) {
        self.section("HEADER");
        self.pair(9, "$ACADVER");
        self.pair(1, ACADVER);
        self.pair(9, "$INSUNITS");
        self.pair(70, 1);
        self.pair(9, "$DWGCODEPAGE");
        self.pair(3, "ANSI_1252");

        let min_x = ext.min_x - PAD_IN;
        let min_y = ext.min_y - PAD_IN;
        let max_x = ext.max_x + PAD_IN;
        let max_y = ext.max_y + PAD_IN;

        self.pair(9, "$EXTMIN");
        self.pair(10, fmt_num(min_x));
        self.pair(20, fmt_num(min_y));
        self.pair(30, "0.0");
        self.pair(9, "$EXTMAX");
        self.pair(10, fmt_num(max_x));
        self.pair(20, fmt_num(max_y));
        self.pair(30, "0.0");
        self.pair(9, "$LIMMIN");
        self.pair(10, fmt_num(min_x));
        self.pair(20, fmt_num(min_y));
        self.pair(9, "$LIMMAX");
        self.pair(10, fmt_num(max_x));
        self.pair(20, fmt_num(max_y));
        self.endsec();
    }

    fn write_layer(&mut self, name: &str, color: i32) {
        self.pair(0, "LAYER");
        let handle = self.next_handle();
        self.pair(5, handle);
        self.pair(330, "2");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbLayerTableRecord");
        self.pair(2, name);
        self.pair(70, 0);
        self.pair(62, color);
        self.pair(370, -3);
    }

    fn write_tables(&mut self) {
        self.section("TABLES");

        self.table("VPORT", 1);
        self.pair(0, "VPORT");
        let handle = self.next_handle();
        self.pair(5, handle);
        self.pair(330, "8");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbViewportTableRecord");
        self.pair(2, "*Active");
        self.endtab();

        self.table("LTYPE", 1);
        self.pair(0, "LTYPE");
        let handle = self.next_handle();
        self.pair(5, handle);
        self.pair(330, "5");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbLinetypeTableRecord");
        self.pair(2, "CONTINUOUS");
        self.pair(70, 0);
        self.pair(3, "Solid line");
        self.pair(72, 65);
        self.pair(73, 0);
        self.pair(40, "0.0");
        self.endtab();

        self.table("LAYER", 5);
        for (name, color) in [
            ("0", 7),
            ("WIRES", 7),
            ("FILLS", 9),
            ("INKFILL", 7),
            ("GUIDES", 8),
        ] {
            self.write_layer(name, color);
        }
        self.endtab();

        self.table("STYLE", 1);
        self.pair(0, "STYLE");
        let handle = self.next_handle();
        self.pair(5, handle);
        self.pair(330, "3");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbTextStyleTableRecord");
        self.pair(2, "Standard");
        self.pair(70, 0);
        self.pair(40, "0.0");
        self.pair(41, "1.0");
        self.pair(50, "0.0");
        self.pair(71, 0);
        self.pair(42, "0.2");
        self.pair(3, "Arial Narrow.ttf");
        self.pair(4, "");
        self.endtab();

        self.table("VIEW", 0);
        self.endtab();

        self.table("UCS", 0);
        self.endtab();

        self.table("APPID", 1);
        self.pair(0, "APPID");
        let handle = self.next_handle();
        self.pair(5, handle);
        self.pair(330, "9");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbRegAppTableRecord");
        self.pair(2, "ACAD");
        self.pair(70, 0);
        self.endtab();

        self.table("DIMSTYLE", 1);
        self.pair(0, "DIMSTYLE");
        let handle = self.next_handle();
        self.pair(105, handle);
        self.pair(330, "A");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbDimStyleTableRecord");
        self.pair(2, "Standard");
        self.pair(70, 0);
        self.endtab();

        self.table("BLOCK_RECORD", 2);
        self.pair(0, "BLOCK_RECORD");
        let model_handle = self.model_space_handle.clone();
        self.pair(5, model_handle);
        self.pair(330, "1");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbBlockTableRecord");
        self.pair(2, "*Model_Space");

        self.pair(0, "BLOCK_RECORD");
        let paper_handle = self.paper_space_handle.clone();
        self.pair(5, paper_handle);
        self.pair(330, "1");
        self.pair(100, "AcDbSymbolTableRecord");
        self.pair(100, "AcDbBlockTableRecord");
        self.pair(2, "*Paper_Space");
        self.endtab();

        self.endsec();
    }

    fn entity_header(&mut self, kind: &str, layer: &str) {
        let handle = self.next_handle();
        let owner = self.model_space_handle.clone();
        self.pair(0, kind);
        self.pair(5, handle);
        self.pair(330, owner);
        self.pair(100, "AcDbEntity");
        self.pair(8, layer);
        self.pair(370, -3);
    }

    fn write_line(
        &mut self,
        layer: &str,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    ) {
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
        for (x, y) in pts {
            self.pair(10, fmt_num(*x));
            self.pair(20, fmt_num(*y));
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
        self.pair(10, fmt_num(x));
        self.pair(20, fmt_num(y));
        self.pair(30, "0.0");
        self.pair(40, fmt_num(height));
        self.pair(1, value);
        self.pair(7, "Standard");
        self.pair(100, "AcDbText");
        self.pair(72, h_align.dxf_code());
        self.pair(11, fmt_num(x));
        self.pair(21, fmt_num(y));
        self.pair(31, "0.0");
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
    }

    fn write_entities(&mut self, entities: &[EntityKind]) {
        self.section("ENTITIES");
        for entity in entities {
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
        self.endsec();
    }

    fn finish(mut self) -> String {
        self.pair(0, "EOF");
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
    let mut writer = DxfWriter::new();
    writer.write_header(doc.extent);
    writer.write_tables();
    writer.write_entities(&doc.entities);
    writer.finish()
}
