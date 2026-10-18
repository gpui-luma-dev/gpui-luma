use super::{MotionOverrides, Transition};

/// Deferred edits to look defaults. Controls freeze this at spawn; animation
/// implementations and frame state never live in this configuration.
#[derive(Clone)]
pub struct MotionConfig<M> {
    profile: M,
    edits: MotionOverrides<M>,
    enabled: bool,
    frozen: bool,
    supplied: bool,
}

impl<M: Default> Default for MotionConfig<M> {
    fn default() -> Self {
        Self { profile: M::default(), edits: Default::default(), enabled: true, frozen: false, supplied: false }
    }
}

impl<M: Clone> MotionConfig<M> {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn configure(&mut self, edit: impl Fn(&mut M) + Send + Sync + 'static) {
        self.edits.push(edit);
        self.frozen = false;
    }
    pub fn defaults(&mut self, profile: M) {
        self.profile = profile;
        self.supplied = true;
        self.frozen = false;
    }
    /// Append explicit edits, reopening a frozen snapshot without reinheriting defaults.
    pub fn overrides(&mut self, edits: MotionOverrides<M>) {
        self.edits.append(edits);
        self.frozen = false;
    }

    pub fn resolve(&self) -> M {
        let mut profile = self.profile.clone();
        self.edits.apply(&mut profile);
        profile
    }
    pub fn freeze(&mut self) {
        self.profile = self.resolve();
        self.edits = Default::default();
        self.frozen = true;
        self.supplied = true;
    }
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
    pub fn inherit_with(&mut self, defaults: impl FnOnce() -> M, edits: MotionOverrides<M>) -> &M {
        if !self.frozen {
            if !self.supplied {
                self.profile = defaults();
            }
            self.edits.append(edits);
            self.freeze();
        }
        &self.profile
    }
    pub fn profile(&self) -> &M {
        &self.profile
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! motion_policy {
    ($($field:ident).+) => {
        /// Disable every motion role without erasing authored configuration.
        pub fn motion_enabled(mut self, enabled: bool) -> Self {
            self.$($field).+.set_enabled(enabled); self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! presence_policy {
    ($($target:ident),+ $(,)?) => {
        pub fn motion_enabled(&self) -> bool { self.model.motion.enabled() }
        /// Settle all presence/disclosure channels immediately when disabled.
        pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut gpui::Context<Self>) {
            self.model.motion.set_enabled(enabled);
            $(self.$target.set_motion_enabled(enabled);)+
            cx.notify();
        }
    };
}

impl Transition {
    /// Final control-wide policy, independent of the selected implementation.
    pub fn enabled(&self, enabled: bool) -> Self {
        if enabled { self.clone() } else { Self::instant() }
    }
}

/// Shared declaration plumbing; controls declare roles, and the SDK stylesheet
/// supplies the vanilla profile values.
#[doc(hidden)]
#[macro_export]
macro_rules! motion_profile {
    ($name:ident => $family:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[doc = "Control-owned motion roles. Looks supply defaults; instance edits replace selected roles."]
        #[derive(Clone, Debug)]
        pub struct $name { $(pub $field: $ty),* }
        impl Default for $name {
            fn default() -> Self { $crate::theme::MotionDefaults::vanilla().$family.clone() }
        }
    };
}

/// The SDK owns resolution; look builders only forward deferred edits.
#[doc(hidden)]
#[macro_export]
macro_rules! motion_methods {
    ($profile:ty, $($field:ident).+) => {
        /// Edit inherited roles after look defaults, independent of setter order.
        pub fn motion(mut self, configure: impl Fn(&mut $profile) + Send + Sync + 'static) -> Self {
            self.$($field).+.configure(configure); self
        }
        /// Supply the complete resolved look profile without replacing consumer edits.
        pub fn motion_defaults(mut self, profile: $profile) -> Self {
            self.$($field).+.defaults(profile); self
        }
        #[doc(hidden)]
        pub fn with_motion_overrides(mut self, edits: $crate::motion::MotionOverrides<$profile>) -> Self {
            self.$($field).+.overrides(edits); self
        }
        /// Authored roles before the control-wide motion policy.
        pub fn motion_profile(&self) -> $profile { self.$($field).+.resolve() }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! forward_motion {
    ($profile:ty, edits $field:ident) => {
        /// Edit the SDK profile after this look's defaults are resolved.
        pub fn motion(mut self, configure: impl Fn(&mut $profile) + Send + Sync + 'static) -> Self {
            self.$field.push(configure);
            self
        }
    };
    ($profile:ty, builder $field:ident) => {
        /// Edit the SDK profile after this look's defaults are resolved.
        pub fn motion(mut self, configure: impl Fn(&mut $profile) + Send + Sync + 'static) -> Self {
            self.$field = self.$field.motion(configure);
            self
        }
    };
}

impl<M: std::fmt::Debug> std::fmt::Debug for MotionConfig<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.profile.fmt(f)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! motion_snapshot {
    ($profile:ty, $($field:ident).+) => {
        /// Motion roles captured when this control was created.
        pub fn motion_profile(&self) -> &$profile { self.$($field).+.profile() }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::tabs::TabsMotion;
    use crate::motion::{Easing, MotionState};
    use std::time::Duration;

    #[test]
    fn appended_edits_thaw_a_snapshot_without_reinheriting_or_replaying_edits() {
        let mut config = MotionConfig::<u32>::default();
        config.configure(|value| *value += 1);
        assert_eq!(*config.inherit_with(|| 10, Default::default()), 11);
        let mut edits = MotionOverrides::default();
        edits.push(|value: &mut u32| *value += 2);
        config.overrides(edits);
        assert!(!config.is_frozen());
        assert_eq!(*config.inherit_with(|| panic!("must retain captured defaults"), Default::default()), 13);
        config.freeze();
        assert_eq!(config.resolve(), 13);
        assert_eq!(*config.inherit_with(|| 100, Default::default()), 13);
    }

    #[test]
    fn inherited_roles_merge_once_and_policy_does_not_erase_configuration() {
        let mut config = MotionConfig::<TabsMotion>::default();
        config.configure(|m| m.indicator = Transition::instant());
        let mut later = MotionOverrides::default();
        later.push(|m: &mut TabsMotion| m.content = Transition::timed(Duration::from_secs(1), Easing::Linear));
        let profile = config.inherit_with(TabsMotion::default, later);
        assert!(profile.indicator.start(MotionState::default(), 1.0).sample(Duration::ZERO).complete);
        assert_eq!(
            profile.content.start(MotionState::default(), 1.0).sample(Duration::from_millis(100)).state.value,
            0.1
        );
        config.set_enabled(false);
        assert!(
            config
                .profile()
                .content
                .enabled(config.enabled())
                .start(MotionState::default(), 1.0)
                .sample(Duration::ZERO)
                .complete
        );
        config.set_enabled(true);
        config.inherit_with(|| panic!("a live control must retain its look snapshot"), MotionOverrides::default());
        assert!(
            !config
                .profile()
                .content
                .enabled(config.enabled())
                .start(MotionState::default(), 1.0)
                .sample(Duration::ZERO)
                .complete
        );
    }
}
