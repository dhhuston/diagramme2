//! Scene graph types — diagram px, authoritative for Konva and DXF (via `scene_to_cad`).

use serde::{Deserialize, Serialize};

pub use diagramme_geometry::{PointPx, RectPx};

/// Horizontal text alignment in diagram space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

/// Vertical text alignment in diagram space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

/// Drawable primitive in diagram pixels (Y-down). DXF consumes `scene_to_cad` only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScenePrimitive {
    Polyline {
        points: Vec<PointPx>,
        stroke_px: f64,
        layer: String,
        color: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        edge_id: Option<String>,
    },
    Rect {
        rect: RectPx,
        stroke_px: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        fill: Option<u32>,
        layer: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        node_id: Option<String>,
    },
    Solid {
        vertices: [PointPx; 4],
        layer: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        node_id: Option<String>,
    },
    Text(SceneText),
}

/// Text primitive — cap height in diagram px is final (no export visual scale).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneText {
    pub position: PointPx,
    pub content: String,
    pub height_px: f64,
    pub halign: HAlign,
    pub valign: VAlign,
    pub font: String,
}

/// Konva hit region in diagram pixels (inverse viewport → diagram → hit id).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HitTarget {
    pub id: String,
    pub bounds: RectPx,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_id: Option<String>,
}

/// Full scene for one sheet — single geometric truth for canvas and export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub primitives: Vec<ScenePrimitive>,
    pub extent: RectPx,
    pub hits: Vec<HitTarget>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            primitives: Vec::new(),
            extent: RectPx::new(0.0, 0.0, 0.0, 0.0),
            hits: Vec::new(),
        }
    }
}
