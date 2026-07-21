# Retired Documents Index

This index organizes historical design plans, draft specifications, and architectural proposals that have been retired from the active development tracks. It serves as a guide for cleaning up the documentation space.

Each retired document is listed below with a summary of its scope, status, and a recommendation on whether to **Keep** (as valuable design context or decision records) or **Delete** (as noise/obsolete drafts).

---

## 📊 Sizing, Density & Layout Sizing

| Document | Description | Recommendation | Rationale |
| :--- | :--- | :---: | :--- |
| [theme-revisit.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/theme-revisit.md) | Specs for decoupling **Layout Scales** (SDK-owned geometry) from **Visual Palettes** (Theme colors). | **KEEP** | Essential design rationale for how Luma handles spatial density and subpixel pixel-snapping. |
| [theme-revisit-plan.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/theme-revisit-plan.md) | Phased execution roadmap for the Phase-5 structural density refactoring. | **DELETE** | Pure project-management roadmap; the refactoring is 100% completed. |
| [deprecate-layout.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/deprecate-layout.md) | Design debt audit targeting leaky dimensions in the global `layout.rs` caching utilities. | **KEEP** | Highlights key lessons about maintaining separation of concerns in control-specific metrics. |
| [future-layout.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/future-layout.md) | Ideation on WPF-style layout containers (`vstack!`, `hstack!`, `wrappanel!`). | **KEEP** | Useful syntax reference for macro design, relevant to the current `wrappanel` and visual layout builders. |
| [future-layout-2.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/future-layout-2.md) | Sizing rules and box constraints analysis modeled after Jetpack Compose and SwiftUI. | **KEEP** | Important reading for any future adjustments to the core layout constraint engine. |

---

## 🗂️ ListView & virtualization

| Document | Description | Recommendation | Rationale |
| :--- | :--- | :---: | :--- |
| [listview-vnext.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/listview-vnext.md) | The target design for the unified grid `list_view!` macro syntax and custom columns. | **DELETE** | Superseded by the completed implementation in the codebase. |
| [listview_2.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/listview_2.md) | Design spec for Phase-2 paging toolbar, snap physics, and visible-rows container sizing. | **DELETE** | All pagination features and macros are fully implemented. |
| [listview_3.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/listview_3.md) | Refactoring spec to unify scrolling/paging facades and compile-time macro syntax. | **DELETE** | Paging facade and `scrolling_list_view!` / `paging_list_view!` macros are finished. |
| [list-control.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/list-control.md) | Pre-Luma initial thoughts on standard virtual list virtualization. | **DELETE** | Obsolete draft that doesn't correspond to the current architecture. |
| [list-control-refine.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/list-control-refine.md) | Early XAML-inspired list helper macro concepts. | **DELETE** | Fully replaced by the explicit macro rollout in `listview_3.md`. |

---

## 🛠️ Custom Controls Specs

| Document | Description | Recommendation | Rationale |
| :--- | :--- | :---: | :--- |
| [treeview-plan.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/treeview-plan.md) | Flat virtualization engine and arrow-navigation specification for `TreeViewControl`. | **KEEP** | Great architectural map explaining how the flattened caching layers map to GPUI's virtualized `ListState`. |
| [accordian-plan.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/accordian-plan.md) | Plan to refactor accordion custom closures and style rows. | **DELETE** | Completed. Accordion control is stable and deployed. |
| [navigation-sidebar.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/navigation-sidebar.md) | Internal structure and focus-coordination model of the Sidebar navigation control. | **KEEP** | Documents the complex routing and node selection propagation logic. |

---

## 🎨 Theming, State-Binding & Experiments

| Document | Description | Recommendation | Rationale |
| :--- | :--- | :---: | :--- |
| [luma-studio.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/luma-studio.md) | Spec sheet and card layout outline for the native customizer tool. | **DELETE** | Completed. The Luma Studio app (`luma-studio`) is fully implemented. |
| [look.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/look.md) | Design proposal for consolidating look parameter signatures. | **KEEP** | Useful reference for signature patterns if we perform another SDK-wide refactoring. |
| [closures.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/closures.md) | Argument for stateless visual elements to avoid double storage and synchronization code. | **KEEP** | Represents a major potential architectural direction that could still be revisited. |
| [form-binding-ideas.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/form-binding-ideas.md) | Exploration of macros vs dispatcher state-sync frameworks. | **KEEP** | Serves as design context for how `declare_form!` was chosen over other patterns. |
| [nav_tree.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/nav_tree.md) | Proposal for an visual `nav_tree!` shorthand macro. | **DELETE** | Unused; builder syntax is readable enough for current hierarchies. |
| [plan-model.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/plan-model.md) | Unified state model proposal for introduction panels. | **DELETE** | The `EventBus` approach was retained; this is a dead-end plan. |
| [gallery-enhancement.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/gallery-enhancement.md) | Draft ideas for mapping token usage tables in the gallery. | **DELETE** | The in-app active usage catalog already handles this natively. |
| [theme-spec-dumper.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/theme-spec-dumper.md) | Scratched plan to dump JSON specs out of active modules using `linkme`. | **DELETE** | Superseded by the in-app Luma Studio inspector. |
| [proto3.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/proto3.md) | Decision spec for migrating stringly-typed metadata in custom controls. | **KEEP** | Historical context on component refactoring and metadata models. |
| [proto3_refactored_example.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/proto3_refactored_example.md) | Accompanying code outline demonstrating type-safe field selectors. | **DELETE** | Outdated code scratchpad. |
| [rhai-thoughts.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/retired/rhai-thoughts.md) | Scratchpad exploring scriptable styling with the Rhai scripting engine. | **DELETE** | Unrelated to the current CSS-driven Radix theme pipeline. |
