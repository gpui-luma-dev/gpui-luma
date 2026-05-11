# OLDER DOCUMENT NOT TO BE CONSIDERED AT THIS POINT

# Selector Composition Target (v2)

## Goal

Build selector-family controls from one shared selector core, then apply bounded specializations.

- Shared core: items, query/filtering, selection, highlight, popup open/close, scroll visibility.
- Specialization points: `trigger`, `popup`, optional `item_presenter`.
- Control wrappers (`combobox`, `search_selector`) should be thin compositions.

## Composition Surface

```/dev/null/selector_core.pseudo.rs#L1-74
pub struct SelectorCoreModel {
    pub items: Vec<String>,
    pub query: String,
    pub selected: Option<usize>,
    pub highlighted_visible: Option<usize>,
    pub popup_open: bool,
}

pub struct SelectorSpecialization {
    pub trigger: TriggerSpec,
    pub popup: PopupSpec,
    pub item_presenter: ItemPresenterSpec,
    pub template: SelectorTemplateSpec,
    pub theme: SelectorThemeSpec,
}

pub struct TriggerSpec {
    pub read_only: bool,
    pub clearable: bool,
    pub open_on_down: bool,
    pub open_on_double_click: bool,
}

pub enum PopupSpec {
    ListOnly,
    SearchAndList { search_placeholder: String },
}

pub enum ItemPresenterSpec {
    LabelText,
    Custom(fn(item: &str) -> String),
}

pub struct SelectorTemplateSpec {
    pub trigger_template: TriggerTemplate,
    pub popup_template: PopupTemplate,
    pub row_template: RowTemplate,
}

pub struct SelectorThemeSpec {
    pub trigger_appearance: TriggerAppearance,
    pub popup_appearance: PopupAppearance,
    pub row_appearance: RowAppearance,
}

pub struct ComposedSelectorControl {
    pub core: SelectorCoreModel,
    pub spec: SelectorSpecialization,
}
```

## Combobox specialization

```/dev/null/combobox_composed.pseudo.rs#L1-44
pub struct ComboboxModel {
    pub trigger_label: String,
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub query: String,
}

pub fn compose_combobox(model: ComboboxModel) -> ComposedSelectorControl {
    ComposedSelectorControl {
        core: SelectorCoreModel {
            items: model.items,
            query: model.query,
            selected: model.selected,
            highlighted_visible: None,
            popup_open: false,
        },
        spec: SelectorSpecialization {
            trigger: TriggerSpec {
                read_only: false,
                clearable: true,
                open_on_down: true,
                open_on_double_click: true,
            },
            popup: PopupSpec::ListOnly,
            item_presenter: ItemPresenterSpec::LabelText,
            template: combobox_templates(),
            theme: combobox_theme(),
        },
    }
}
```

## SearchSelector specialization

```/dev/null/search_selector_composed.pseudo.rs#L1-46
pub struct SearchSelectorModel {
    pub trigger_label: String,
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub search_placeholder: String,
}

pub fn compose_search_selector(model: SearchSelectorModel) -> ComposedSelectorControl {
    ComposedSelectorControl {
        core: SelectorCoreModel {
            items: model.items,
            query: String::new(),
            selected: model.selected,
            highlighted_visible: None,
            popup_open: false,
        },
        spec: SelectorSpecialization {
            trigger: TriggerSpec {
                read_only: true,
                clearable: false,
                open_on_down: true,
                open_on_double_click: false,
            },
            popup: PopupSpec::SearchAndList {
                search_placeholder: model.search_placeholder,
            },
            item_presenter: ItemPresenterSpec::LabelText,
            template: search_selector_templates(),
            theme: search_selector_theme(),
        },
    }
}
```

## Code-verified feature catalog (current)

### Autocomplete
- Trigger is textfield-like; user types query directly.
- Has clear `x` affordance.
- Popup list opens while focused + query has matches (scroll surface enabled).
- Keyboard highlight movement: up/down.
- Events: `Change`, `Select`, `Complete`, `Clear`.
- Note: no dedicated typing restriction policy field in current autocomplete model.

### Combobox
- Trigger is textfield-like; type-to-search.
- Down key opens full items popup.
- Has clear `x` and dropdown chevron affordances.
- Popup is scrolling list with highlight movement (up/down/page/home/end).
- Events: `Change`, `Select`, `Complete`, `Clear`.
- Note: `TypingPolicy` exists in model (`Flexible`/`Strict`), but current control path does not yet apply it as hard input restriction logic.

### SearchSelector
- Trigger is selector-like (read-only style); click/focus opens popup.
- Popup includes dedicated search textfield + scrolling results list.
- Keyboard highlight movement includes up/down/page/home/end.
- Events: `Change`, `Select`, `Complete`, `Clear`.

### Selector
- Trigger is selector-style with dropdown arrow; no free text entry.
- Selection is from popup options only.
- Supports presenter-style customization via item template (`with_item_template(...)`) and panel template.
- Best for small-to-medium option sets (not hard-enforced by model).

## Template and theme considerations

- Keep one shared selector template contract (`trigger`, `popup`, `row` regions).
- Allow per-control template overrides only through specialization config, not separate per-control template stacks.
- Keep one shared selector appearance surface (trigger/popup/row); controls provide only controlled overrides.
- Theme usage metadata should be owned by each composed control, but mapped through shared selector appearance fields.
- Default item presenter is plain text label; custom presenter remains optional.
