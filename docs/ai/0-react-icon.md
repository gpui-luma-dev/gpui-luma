# Luma Radix Icon Inventory

The `luma-radix` app needs the following Radix Icons. The app is incomplete, but this inventory is considered accurate. Source the SVG assets from `~/Downloads/radix-icons` and include them with the app assets.

## Navigation and global controls

| Icon | Usage |
| --- | --- |
| `GitHubLogoIcon` | Link to the repository in the top navigation bar; also used by the GitHub social sign-in preview. |
| `SunIcon` | Light-theme side of the theme toggle. |
| `MoonIcon` | Dark-theme side of the theme toggle. |
| `ChevronDownIcon` | Dropdown menus in the global header and mobile navigation drawer. |
| `CaretDownIcon` | Alternate dropdown indicator for the global header and mobile navigation drawer. |

## Custom palette generator and tools

| Icon | Usage |
| --- | --- |
| `CopyIcon` | Copy generated CSS variables or hex scales to the clipboard. |
| `CheckIcon` | Replaces `CopyIcon` as the copied-value feedback indicator; also prefixes tag pills. |
| `ResetIcon` | Reset sliders and custom inputs to their default palette values. |
| `ReloadIcon` | Alternate reset/reload affordance for restoring default palette values. |

Slider thumbs and handles are native slider indicators, not separate icon assets.

## Radix-to-Lucide cross-match

These mappings are based on the repository's pinned `lucide-static-svg` icon enum.

| Radix icon | Lucide equivalent | Match |
| --- | --- | --- |
| `GitHubLogoIcon` | None | Lucide does not include brand logos; retain the Radix SVG. |
| `SunIcon` | `Sun` | Exact |
| `MoonIcon` | `Moon` | Exact |
| `ChevronDownIcon` | `ChevronDown` | Exact |
| `CaretDownIcon` | `ChevronDown` | Closest; no caret equivalent. |
| `CopyIcon` | `Copy` | Exact |
| `CheckIcon` | `Check` | Exact |
| `ResetIcon` | `RotateCcw` | Closest semantic match. |
| `ReloadIcon` | `RefreshCw` or `RefreshCcw` | Closest semantic match. |
| `InfoCircledIcon` | `Info` | Closest. |
| `TextAlignLeftIcon` | `TextAlignStart` | Closest. |
| `TextAlignCenterIcon` | `TextAlignCenter` | Exact |
| `TextAlignRightIcon` | `TextAlignEnd` | Closest. |
| `DotsHorizontalIcon` | `Ellipsis` | Exact. |
| `QuoteIcon` | `Quote` | Exact. |
| `SpeakerLoudIcon` | `Volume2` | Closest. |
| `SpeakerQuietIcon` | `Volume1` | Closest. |
| `EnvelopeClosedIcon` | `Mail` | Closest. |
| `LockClosedIcon` | `Lock` | Closest. |
| `EyeOpenIcon` | `Eye` | Exact. |
| `EyeNoneIcon` | `EyeOff` | Closest. |
| `Cross2Icon` | `X` | Closest. |

`GitHubLogoIcon` is the only listed icon without a Lucide equivalent. The caret and other closest matches should remain Radix when preserving the original Radix visual language matters.

## Component preview canvas

### Card 1: Status and actions

| Icon | Usage |
| --- | --- |
| `InfoCircledIcon` | Leading icon in the “Please upgrade to the new version” alert banner. |
| `TextAlignLeftIcon` | Left option in the text-alignment toggle group. |
| `TextAlignCenterIcon` | Center option in the text-alignment toggle group. |
| `TextAlignRightIcon` | Right option in the text-alignment toggle group. |
| `CheckIcon` | Prefix badge icon on “Fully-featured”, “Built with Radix”, and “Open source” tag pills. |
| `ChevronDownIcon` | Alternate options trigger icon on Emily Adams’s user card. |
| `DotsHorizontalIcon` | Alternate options trigger icon on Emily Adams’s user card. |

The text-alignment controls use `role="group"`.

### Card 2: Content and layout

| Icon | Usage |
| --- | --- |
| `QuoteIcon` | Decorative pull-quote indicator for the Susan Kare design quote. |
| `SpeakerLoudIcon` | Loud/maximum end of the volume or level sliders. |
| `SpeakerQuietIcon` | Quiet/minimum end of the volume or level sliders. |

Contrast or brightness indicators may be used instead of the speaker icons if the preview is presented as a visual-level control rather than volume.

### Card 3: Authentication modal preview

| Icon | Usage |
| --- | --- |
| `EnvelopeClosedIcon` | Email-field input adornment. |
| `LockClosedIcon` | Password-field input adornment. |
| `EyeOpenIcon` | Show-password visibility toggle state. |
| `EyeNoneIcon` | Hide-password visibility toggle state. |
| `GitHubLogoIcon` | “Continue with GitHub” social sign-in button. |
| `Cross2Icon` | Dialog close button in the top-right corner. |
