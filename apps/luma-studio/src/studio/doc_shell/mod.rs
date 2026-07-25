mod sticky_section_heading;

pub(crate) use sticky_section_heading::{
    render_section_heading_anchor, render_section_heading_anchor_with_order_options,
    render_sticky_section_heading_lane, with_sticky_heading_tracker, StickySectionHeadingTracker,
    SECTION_HEADING_CONTENT_GAP, SECTION_HEADING_SHELL_PAD_BOTTOM, SECTION_HEADING_SHELL_PAD_TOP,
};
