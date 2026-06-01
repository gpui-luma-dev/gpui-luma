use std::sync::Arc;

use gpui::{AppContext, Context, Entity, Subscription};
use gpui_luma::theme::{ControlSize, RadixTheme};

use super::app::ThemeStudioApp;
use super::panels::{AccountPanel, ChatPanel, CookiesPanel, PaymentsPanel, ReportPanel, TeamPanel, UpgradePanel};

pub struct DemoControls {
    pub upgrade: Entity<UpgradePanel>,
    pub account: Entity<AccountPanel>,
    pub team: Entity<TeamPanel>,
    pub chat: Entity<ChatPanel>,
    pub cookies: Entity<CookiesPanel>,
    pub report: Entity<ReportPanel>,
    pub payments: Entity<PaymentsPanel>,
}

impl DemoControls {
    pub fn spawn(cx: &mut Context<ThemeStudioApp>, theme: Arc<RadixTheme>, size: ControlSize) -> Self {
        Self {
            upgrade: cx.new(|cx| UpgradePanel::new(cx, theme.clone(), size)),
            account: cx.new(|cx| AccountPanel::new(cx, theme.clone(), size)),
            team: cx.new(|cx| TeamPanel::new(cx, theme.clone())),
            chat: cx.new(|cx| ChatPanel::new(cx, theme.clone(), size)),
            cookies: cx.new(|cx| CookiesPanel::new(cx, theme.clone())),
            report: cx.new(|cx| ReportPanel::new(cx, theme.clone(), size)),
            payments: cx.new(|cx| PaymentsPanel::new(cx, theme.clone(), size)),
        }
    }

    pub fn subscribe(&self, _cx: &mut Context<ThemeStudioApp>, _subscriptions: &mut Vec<Subscription>) {}
}
