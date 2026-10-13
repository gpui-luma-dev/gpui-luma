//! Tab-entry reveal shared by example hosts; replay transitions remain independent.
use gpui::{App, Entity};
pub(crate) trait Entrance {
    fn set_entrance(&mut self, progress: f32) -> bool;
}
pub(crate) fn display<T: Entrance + 'static>(entity: &Entity<T>, progress: f32, cx: &mut App) {
    entity.update(cx, |host, cx| {
        if host.set_entrance(progress) {
            cx.notify();
        }
    });
}
macro_rules! entrance {
    ($host:ty) => {
        impl crate::examples::motion::Entrance for $host {
            fn set_entrance(&mut self, progress: f32) -> bool {
                let changed = self.entrance != progress;
                self.entrance = progress;
                changed
            }
        }
    };
}
pub(crate) use entrance;
