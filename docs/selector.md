# Selection Component Selection Guide

This guide explains the "Why" behind our three core selection components. While they share a common internal engine, each is optimized for a specific **User Intent**.

---

## 1. The Selector (The "Safe" Choice)
**The Story:** "Use this when the user must choose from a small, fixed set of options, and you want to prevent them from even trying to type something else."

* **Behavior:** Closed-list, no keyboard input (except for jumping to a letter), mouse-driven.
* **The "Why":** It eliminates "Input Error" entirely. Perfect for "Yes/No," "Gender," or "Shipping Method."

---

## 2. The Combobox (The "Efficient" Choice)
**The Story:** "Use this when there is a correct answer in a long list, and you want to help the user find it without scrolling for ten minutes."

* **Behavior:** Filterable list, "Down Arrow" for discovery, but **Strict Validation**. If it's not in the list, it's not in the box.
* **The "Why":** It balances the power of a Search bar with the data integrity of a Dropdown. Perfect for "Country," "Employee Name," or "Part Number."

---

## 3. The Autocomplete (The "Helpful" Choice)
**The Story:** "Use this when the user is free to type whatever they want, but you want to save them keystrokes by suggesting common entries."

* **Behavior:** Suggestion pane only, no "Down Arrow" (usually), **Loose Validation**. The user owns the text; you just provide the shortcuts.
* **The "Why":** It’s about speed and "Search" rather than "Selection." Perfect for "Job Title," "Search Bars," or "Tags."

---

### Comparison Summary

| Component | UI Strategy | Validation Policy | Best For... |
| :--- | :--- | :--- | :--- |
| **Select** | Mouse/Selection | Absolute (Locked) | Small, fixed lists |
| **Combobox** | Filter & Pick | Strict (Clear on Blur) | Large, required lists |
| **Autocomplete** | Text Entry | Loose (Suggestions) | Open-ended text |