use super::*;
use gpui_luma::infra::attachments::TooltipEntityExt;
use gpui_luma::controls::tooltip::Tooltip;

pub(super) fn default_points() -> Vec<MeshPoint> {
    let [yellow, pink, blue] = super::mesh::default_point_palette();
    [(0.82, 0.90, blue), (0.30, 0.28, yellow), (0.38, 0.46, pink), (0.08, 0.12, pink)]
        .into_iter()
        .enumerate()
        .map(|(index, (u, v, color))| MeshPoint { row: 0, col: index as u8, u, v, color })
        .collect()
}

impl GradientBuilder {
    pub(super) fn init_freeform_buttons(&mut self, cx: &mut Context<Self>) {
        for (index, (icon, label)) in [
            (LucideIcon::Plus, "Add point"),
            (LucideIcon::Trash, "Remove selected point"),
            (LucideIcon::Shuffle, "Randomize point positions"),
            (LucideIcon::Palette, "Randomize selected point color"),
        ]
        .into_iter()
        .enumerate()
        {
            let button = shadcn::Button::icon_button(format!("color-viz-freeform-{index}"), icon)
                .look(self.look.as_ref())
                .outline()
                .spawn(cx)
                .tooltip(Tooltip::new(label), cx);
            self._subscriptions.push(cx.subscribe(&button, move |this, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    this.edit_freeform(index, cx);
                }
            }));
            self.freeform_buttons.push(button);
        }
    }

    pub(super) fn sync_freeform_buttons(&mut self, cx: &mut Context<Self>) {
        self.freeform_buttons[0].update(cx, |button, cx| button.set_enabled(self.mesh_points.len() < 16, cx));
        self.freeform_buttons[1].update(cx, |button, cx| {
            button.set_enabled(self.mesh_points.len() > 1 && self.selected_mesh_point.is_some(), cx)
        });
        self.freeform_buttons[3].update(cx, |button, cx| button.set_enabled(self.selected_mesh_point.is_some(), cx));
    }

    pub(super) fn sync_point_spread(&mut self, cx: &mut Context<Self>) {
        if self.selected_tab == BuilderTab::Freeform {
            let value =
                self.selected_mesh_point.and_then(|index| self.point_spreads.get(index)).copied().unwrap_or(100.0);
            self.point_spread_slider.update(cx, |slider, cx| {
                slider.set_value(value, cx);
                slider.set_enabled(self.selected_mesh_point.is_some(), cx);
            });
            self.sync_freeform_buttons(cx);
        }
    }

    pub(super) fn deselect_freeform_point(&mut self, cx: &mut Context<Self>) {
        if self.selected_tab != BuilderTab::Freeform {
            return;
        }
        self.selected_mesh_point = None;
        self.mesh_color_target = MeshColorTarget::Background;
        self.active_mesh_drag = None;
        self.color_picker_open = false;
        self.sync_point_spread(cx);
        cx.notify();
    }

    fn random_unit(&mut self) -> f32 {
        self.random_seed ^= self.random_seed << 13;
        self.random_seed ^= self.random_seed >> 7;
        self.random_seed ^= self.random_seed << 17;
        (self.random_seed >> 40) as f32 / 16_777_216.0
    }

    fn random_color(&mut self) -> gpui::Hsla {
        gpui::hsla(self.random_unit(), 0.86 + self.random_unit() * 0.14, 0.49 + self.random_unit() * 0.09, 1.0)
    }

    fn edit_freeform(&mut self, action: usize, cx: &mut Context<Self>) {
        if self.selected_tab != BuilderTab::Freeform {
            return;
        }
        match action {
            0 if self.mesh_points.len() < 16 => {
                let point = MeshPoint {
                    row: 0,
                    col: self.mesh_points.len() as u8,
                    u: self.random_unit(),
                    v: self.random_unit(),
                    color: self.random_color(),
                };
                self.mesh_points.push(point);
                self.point_spreads.push(100.0);
                self.selected_mesh_point = Some(self.mesh_points.len() - 1);
            }
            1 if self.mesh_points.len() > 1 => {
                if let Some(index) = self.selected_mesh_point.filter(|index| *index < self.mesh_points.len()) {
                    self.mesh_points.remove(index);
                    self.point_spreads.remove(index);
                    for (index, point) in self.mesh_points.iter_mut().enumerate() {
                        point.col = index as u8;
                    }
                    self.selected_mesh_point = Some(index.min(self.mesh_points.len() - 1));
                }
            }
            2 => {
                for index in 0..self.mesh_points.len() {
                    let u = self.random_unit();
                    let v = self.random_unit();
                    self.mesh_points[index].u = u;
                    self.mesh_points[index].v = v;
                }
            }
            3 => {
                let color = self.random_color();
                if let Some(point) = self.selected_mesh_point.and_then(|index| self.mesh_points.get_mut(index)) {
                    point.color = color;
                }
            }
            _ => return,
        }
        self.sync_point_spread(cx);
        self.sync_freeform_buttons(cx);
        self.mesh_color_target =
            self.selected_mesh_point.map(MeshColorTarget::Point).unwrap_or(MeshColorTarget::Background);
        self.active_mesh_drag = None;
        self.color_picker_open = false;
        self.mesh_swatch_bounds.clear();
        let _ = self.ensure_preview_image_cache(self.preview_size, cx);
        cx.notify();
    }
}
