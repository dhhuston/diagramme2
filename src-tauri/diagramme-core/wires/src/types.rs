//! Wire geometry types aligned with v6 `wireSharpPolyline.ts` / `wireGeometryModel.ts`.

/// Diagram-space point (same convention as v6 `FlowXY`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowXY {
    pub x: f64,
    pub y: f64,
}

impl From<diagramme_geometry::PointPx> for FlowXY {
    fn from(p: diagramme_geometry::PointPx) -> Self {
        Self { x: p.x, y: p.y }
    }
}

/// Outward wire direction from a handle (mirrors React Flow `Position`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Provenance of a wire polyline (mirrors v6 `WirePolylineSource`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WirePolylineSource {
    Canvas,
    FallbackAnalytical,
    FallbackPersisted,
    Missing,
}

/// Result of resolving a sharp polyline for one edge.
#[derive(Debug, Clone, PartialEq)]
pub struct WirePolylineResult {
    pub polyline: Vec<FlowXY>,
    pub source: WirePolylineSource,
}

/// Axis-aligned obstacle rectangle for routing (mirrors v6 `WireObstacleBox`).
#[derive(Debug, Clone, PartialEq)]
pub struct WireObstacleBox {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Stub endpoints `S`, `S1`, `T1`, `T` used by schematic routing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StubEndpoints {
    pub s: FlowXY,
    pub s1: FlowXY,
    pub t1: FlowXY,
    pub t: FlowXY,
    pub stub: f64,
}

/// Quarter-circle fillet corner on an orthogonal polyline (mirrors v6 `SchematicFilletCorner`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchematicFilletCorner {
    pub corner: FlowXY,
    pub p: FlowXY,
    pub q: FlowXY,
    pub sweep: u8,
}

/// DXF wire polyline input to postprocess (mirrors v6 `DxfWirePolylineRecord`).
#[derive(Debug, Clone, PartialEq)]
pub struct DxfWirePolylineRecord {
    pub edge_id: String,
    pub points: Vec<FlowXY>,
    pub is_schematic: bool,
    pub is_bundle: bool,
    pub source_node_id: Option<String>,
}

/// Line segments and arc segments for Revit DXF emit (mirrors v6 `RevitDxfWirePiece`).
#[derive(Debug, Clone, PartialEq)]
pub enum RevitDxfWirePiece {
    Polyline {
        points: Vec<FlowXY>,
        is_bundle: bool,
    },
    FilletArc {
        arc: SchematicFilletCorner,
    },
}

/// Per-edge wire geometry record with DXF-ready pieces.
#[derive(Debug, Clone, PartialEq)]
pub struct WireGeometryEdgeRecord {
    pub edge_id: String,
    pub source_node_id: String,
    pub sharp_polyline: Vec<FlowXY>,
    pub polyline_source: WirePolylineSource,
    pub is_schematic: bool,
    pub is_bundle: bool,
    pub dxf_pieces: Vec<RevitDxfWirePiece>,
}

/// Full wire geometry model (mirrors v6 `WireGeometryModel`).
#[derive(Debug, Clone, PartialEq)]
pub struct WireGeometryModel {
    pub edges: std::collections::HashMap<String, WireGeometryEdgeRecord>,
    pub dxf_pieces: Vec<RevitDxfWirePiece>,
}
