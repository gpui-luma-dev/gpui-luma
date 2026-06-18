use std::sync::Arc;

use gpui::{AppContext, Context, Entity, Subscription};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use super::app::ThemeStudioApp;
use super::panels::{
    AccordionPanel, AccountPanel, ChatPanel, CookiesPanel, DashboardPanel, PaymentsPanel, ReportPanel,
    SystemPreferencesPanel, TeamPanel, TreeViewPanel, UpgradePanel,
};

#[derive(Clone)]
pub struct DemoControls {
    pub upgrade: Entity<UpgradePanel>,
    pub account: Entity<AccountPanel>,
    pub team: Entity<TeamPanel>,
    pub chat: Entity<ChatPanel>,
    pub cookies: Entity<CookiesPanel>,
    pub report: Entity<ReportPanel>,
    pub payments: Entity<PaymentsPanel>,
    pub tree_view: Entity<TreeViewPanel>,
    pub accordion: Entity<AccordionPanel>,
    pub system_preferences: Entity<SystemPreferencesPanel>,
    pub dashboard: Entity<DashboardPanel>,
}

impl DemoControls {
    pub fn spawn(cx: &mut Context<ThemeStudioApp>, look: Arc<ShadcnLook>, size: ControlSize) -> Self {
        Self {
            upgrade: cx.new(|cx| UpgradePanel::new(cx, look.clone(), size)),
            account: cx.new(|cx| AccountPanel::new(cx, look.clone(), size)),
            team: cx.new(|cx| TeamPanel::new(cx, look.clone())),
            chat: cx.new(|cx| ChatPanel::new(cx, look.clone(), size)),
            cookies: cx.new(|cx| CookiesPanel::new(cx, look.clone())),
            report: cx.new(|cx| ReportPanel::new(cx, look.clone(), size)),
            payments: cx.new(|cx| PaymentsPanel::new(cx, look.clone(), size)),
            tree_view: cx.new(|cx| TreeViewPanel::new(cx, look.clone())),
            accordion: cx.new(|cx| AccordionPanel::new(cx, look.clone())),
            system_preferences: cx.new(|cx| SystemPreferencesPanel::new(cx, look.clone(), 72.0)),
            dashboard: cx.new(|cx| DashboardPanel::new(cx, look.clone())),
        }
    }

    pub fn subscribe(&self, _cx: &mut Context<ThemeStudioApp>, _subscriptions: &mut Vec<Subscription>) {}
}
