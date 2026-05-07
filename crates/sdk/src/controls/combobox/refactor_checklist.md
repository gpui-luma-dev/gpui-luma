# ComboBox refactor behavior checklist

Use this checklist after each staged architecture change to verify behavior parity.

## Trigger and popup
- Clicking trigger toggles popup open/close.
- Arrow-down key opens popup when closed.
- Clicking outside closes popup.
- Popup opens with all items when expected.

## Filtering and typing
- Query updates filtering incrementally.
- `TypingPolicy::Flexible` allows partial match workflow.
- `TypingPolicy::Strict` preserves exact match behavior.
- No-match state is surfaced consistently.

## Keyboard navigation
- Arrow up/down moves highlighted item.
- Home/End jump to first/last visible match.
- PageUp/PageDown move by page.
- Enter commits highlighted/exact match behavior.
- Escape clears/close behavior remains unchanged.

## Focus + blur policy
- Blur commits complete selection when valid.
- Blur reverts to committed selection when query is invalid.
- Blur clears when no valid/committed selection exists.
- Focus entry behavior (including deferred open) stays unchanged.

## Events
- `ComboBoxEvent::Change` on query change.
- `ComboBoxEvent::Select` on non-exact selection commit.
- `ComboBoxEvent::Complete` on exact completion commit.
- `ComboBoxEvent::Clear` on clear actions.

## Scroll/highlight
- Highlighted row is kept visible while navigating.
- Scrollbar sync and wheel scrolling behavior remain stable.
- Popup height/row visibility limits (`min_visible_rows`/`max_visible_rows`) remain correct.

## Architecture completion checks
- Control no longer shapes display rows from source items.
- Panel template owns popup shell placement/chrome.
- Items template renders row content from source item + visible/source index mapping.
- Builder/runtime support control + panel + item template hooks.
