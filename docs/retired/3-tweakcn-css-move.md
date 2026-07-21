# Issue #3: Embed Tweakcn CSS files into look-shadcn and expose API

## Description
The built-in `tweakcn` CSS styles are currently stored under the gallery application project path (`apps/gallery/tweakcn/`) and loaded dynamically from the filesystem. Because styling resources are look-specific, they should belong in the downstream `gpui-luma-look-shadcn` theme crate rather than the showcase app. To prevent apps from accessing the local filesystem, the theme crate should compile/embed these CSS files directly and expose helper functions to query them.

## Proposed Solution
1. Move the `tweakcn/` directory containing CSS stylesheets to the assets directory of `crates/look-shadcn`.
2. Embed the CSS stylesheets directly into the `gpui-luma-look-shadcn` binary/library (using `rust-embed` or standard `include_str!`).
3. Expose a public API in the `gpui-luma-look-shadcn` crate to interact with the built-in themes, which automatically normalizes stylesheet file/slug names to human-readable strings:
   - `pub fn list_built_in_themes() -> Vec<String>`: Returns a list of normalized, human-readable theme names (e.g. `foo.css` is normalized to `"Foo"`, and `amber-minimal.css` is normalized to `"Amber Minimal"`).
   - `pub fn read_built_in_theme(name: &str) -> Option<&'static str>`: Resolves the CSS content of the specified theme by matching either its normalized name or its original slug.
4. Update look loader mechanisms in both the `gallery` and `luma-studio` apps to use these functions instead of raw file system accesses.

## Tasks
- [ ] Move CSS stylesheets from `apps/gallery/tweakcn/` to `crates/look-shadcn/assets/tweakcn/`.
- [ ] Embed the CSS files in `crates/look-shadcn` (e.g. using `rust-embed` or a compile-time static mapping of `include_str!`).
- [ ] Implement a helper function to normalize theme file/slug names (e.g. mapping kebab-case to Title Case, stripping `.css` extensions).
- [ ] Implement `list_built_in_themes()` and `read_built_in_theme(...)` in the `gpui-luma-look-shadcn` crate using the normalization helper.
- [ ] Refactor the theme loaders in [theme.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/theme.rs) to use the new look-shadcn list/read APIs.
- [ ] Refactor style references/exporters in [export.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/export.rs).

## Acceptance Criteria
- Gallery and Luma Studio applications retrieve and load built-in theme CSS entirely in-memory using the new look-shadcn APIs.
- No direct file system reads are performed by the applications to load default/built-in CSS theme sheets.
- Theme list names are presented cleanly as human-readable title-cased values (e.g. "Amber Minimal").
