//! Per-node-type scene primitive emitters (mirrors v6 `revitDxfNodeDetail.ts` dispatch).

pub(crate) mod emit;

pub mod antenna;
pub mod av_plate;
pub mod device_v2;
pub mod flyoff_note;
pub mod grouping_zone;
pub mod junction;
pub mod mic_block;
pub mod patch_panel;
pub mod speaker_block;
pub mod text_block;
pub mod volume_control;
pub mod wiretag;

pub use antenna::append_antenna_scene;
pub use av_plate::append_av_plate_scene;
pub use device_v2::append_device_v2_scene;
pub use flyoff_note::append_flyoff_note_scene;
pub use grouping_zone::append_grouping_zone_scene;
pub use junction::append_junction_scene;
pub use mic_block::append_mic_block_scene;
pub use patch_panel::append_patch_panel_scene;
pub use speaker_block::append_speaker_block_scene;
pub use text_block::append_text_block_scene;
pub use volume_control::append_volume_control_scene;
pub use wiretag::{
    append_wiretag_scene, resolve_pair_main_display_text, wiretag_scene_bounds,
};
