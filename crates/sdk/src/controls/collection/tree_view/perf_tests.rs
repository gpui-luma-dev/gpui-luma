use super::*;
use crate::perf_measure::measure;

#[test]
#[ignore = "manual headless performance measurement"]
fn measure_tree_hot_paths() {
    for count in [100, 10_000] {
        let items: Vec<_> = (0..count / 10)
            .map(|branch| {
                TreeNode::new(format!("branch-{branch}"), "Branch", ())
                    .expanded(true)
                    .children((0..9).map(|row| TreeNode::new(format!("row-{branch}-{row}"), "Row", ())))
            })
            .collect();
        measure(&format!("Tree/index rebuild/{count}"), 20, || super::super::state::TreeIndex::new(&items).unwrap());
        let mut app = gpui::TestAppContext::single();
        let (tree, cx) = app.add_window_view(|_, cx| {
            TreeViewControl::from_builder(
                TreeViewBuilder::new("perf-tree").items(items.clone()).motion_enabled(false),
                cx,
            )
        });
        cx.run_until_parked();
        measure(&format!("Tree/headless warm redraw/{count}"), 5, || {
            tree.update(cx, |_, cx| cx.notify());
            cx.run_until_parked();
        });
        measure(&format!("Tree/replace+headless redraw/{count}"), 5, || {
            tree.update(cx, |tree, cx| tree.replace_items(items.clone(), cx).unwrap());
            cx.run_until_parked();
        });
    }
}
