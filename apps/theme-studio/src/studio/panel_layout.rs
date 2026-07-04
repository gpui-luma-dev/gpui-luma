use super::inspectable::InspectableId;
use super::prototypes::column_layout::TileHeightClass;

pub const GAP: f32 = 16.0;

pub fn panel_height_class(id: InspectableId) -> TileHeightClass {
    match id {
        InspectableId::CreateAccount | InspectableId::Chat => TileHeightClass::Compact,
        InspectableId::Payments | InspectableId::UpgradeSubscription => TileHeightClass::Wide,
        _ => TileHeightClass::Normal,
    }
}
