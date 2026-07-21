mod board;

use super::super::super::inspectable::InspectableId;
use super::super::super::prototypes::column_layout::TileHeightClass;

pub use board::render_demo_board;

pub const GAP: f32 = 16.0;
pub const COLUMN_MIN_WIDTH: f32 = 280.0;
pub const COLUMN_PREFERRED_WIDTH: f32 = 300.0;

pub fn panel_height_class(id: InspectableId) -> TileHeightClass {
    match id {
        InspectableId::CreateAccount | InspectableId::Chat => TileHeightClass::Compact,
        InspectableId::Payments | InspectableId::UpgradeSubscription => TileHeightClass::Wide,
        _ => TileHeightClass::Normal,
    }
}
