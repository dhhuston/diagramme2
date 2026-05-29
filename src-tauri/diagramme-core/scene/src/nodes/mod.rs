//! Per-node-type scene primitive emitters (mirrors v6 `revitDxfNodeDetail.ts` dispatch).

pub mod av_plate;
pub mod device_v2;
pub mod patch_panel;

pub use av_plate::append_av_plate_scene;
pub use device_v2::append_device_v2_scene;
pub use patch_panel::append_patch_panel_scene;
