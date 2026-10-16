// Shared desktop and mobile navigation. No fetch or build step; works on file://.
(() => {
    const sections = [
    {
        "title": "Overview",
        "items": [
            {
                "href": "index.html",
                "label": "Getting Started"
            }
        ]
    },
    {
        "title": "Looks",
        "items": [
            {
                "href": "looks.html",
                "label": "Why? How?"
            },
            {
                "href": "look-shadcn.html",
                "label": "Look-shadcn"
            },
            {
                "href": "look-radix.html",
                "label": "Look-radix"
            },
            {
                "href": "create-look.html",
                "label": "Create your own Look"
            }
        ]
    },
    {
        "title": "Controls",
        "items": [
            {
                "href": "accordion.html",
                "label": "Accordion"
            },
            {
                "href": "autocomplete.html",
                "label": "Autocomplete"
            },
            {
                "href": "button.html",
                "label": "Button"
            },
            {
                "href": "checkbox.html",
                "label": "Checkbox"
            },
            {
                "href": "combobox.html",
                "label": "ComboBox"
            },
            {
                "href": "context-menu.html",
                "label": "Context Menu"
            },
            {
                "href": "control-group.html",
                "label": "Control Group"
            },
            {
                "href": "dock-splitter.html",
                "label": "Dock Splitter"
            },
            {
                "href": "icon-button.html",
                "label": "Icon Button"
            },
            {
                "href": "icon-group.html",
                "label": "Icon Group"
            },
            {
                "href": "listbox.html",
                "label": "List Box"
            },
            {
                "href": "overlay-window.html",
                "label": "Overlay Window"
            },
            {
                "href": "pager.html",
                "label": "Pager"
            },
            {
                "href": "popover-button.html",
                "label": "Popover Button"
            },
            {
                "href": "popup-menu.html",
                "label": "Popup Menu"
            },
            {
                "href": "progress.html",
                "label": "Progress"
            },
            {
                "href": "radio-button.html",
                "label": "Radio Button"
            },
            {
                "href": "radio-group.html",
                "label": "Radio Group"
            },
            {
                "href": "resizable-panels.html",
                "label": "Resizable Panels"
            },
            {
                "href": "scrollbar.html",
                "label": "Scrollbar"
            },
            {
                "href": "search-selector.html",
                "label": "Search Selector"
            },
            {
                "href": "selection-panel.html",
                "label": "Selection Panel"
            },
            {
                "href": "selector.html",
                "label": "Selector"
            },
            {
                "href": "sidebar.html",
                "label": "Sidebar"
            },
            {
                "href": "slide-panel.html",
                "label": "Slide Panel"
            },
            {
                "href": "slider.html",
                "label": "Slider"
            },
            {
                "href": "split-button.html",
                "label": "Split Button"
            },
            {
                "href": "split-view.html",
                "label": "Split View"
            },
            {
                "href": "stepper.html",
                "label": "Stepper"
            },
            {
                "href": "switch.html",
                "label": "Switch"
            },
            {
                "href": "table.html",
                "label": "Table"
            },
            {
                "href": "tabs.html",
                "label": "Tabs"
            },
            {
                "href": "textarea.html",
                "label": "Text Area"
            },
            {
                "href": "textfield.html",
                "label": "Text Field"
            },
            {
                "href": "toggle.html",
                "label": "Toggle"
            },
            {
                "href": "toolbar.html",
                "label": "Toolbar"
            },
            {
                "href": "tooltip.html",
                "label": "Tooltip"
            },
            {
                "href": "tree-view.html",
                "label": "Tree View"
            }
        ]
    },
    {
        "title": "Layout",
        "items": [
            {
                "href": "layout.html",
                "label": "Primitives & macros"
            },
            {
                "href": "frame.html",
                "label": "Frame"
            },
            {
                "href": "scroll-container.html",
                "label": "Scroll Container"
            }
        ]
    },
    {
        "title": "Composition & helpers",
        "items": [
            {
                "href": "composition.html",
                "label": "Overview & forms"
            },
            {
                "href": "control-specialization.html",
                "label": "Create a control specialization"
            },
            {
                "href": "floating-menu.html",
                "label": "Floating Menu"
            },
            {
                "href": "popup-scroll-surface.html",
                "label": "Popup Scroll Surface"
            },
            {
                "href": "selector-list.html",
                "label": "Selector List"
            }
        ]
    },
    {
        "title": "Plots",
        "items": [
            {
                "href": "plots.html",
                "label": "Getting started"
            },
            {
                "href": "plot-cartesian.html",
                "label": "Cartesian charts"
            },
            {
                "href": "plot-pie.html",
                "label": "Pie & donut"
            },
            {
                "href": "plot-radar.html",
                "label": "Radar charts"
            },
            {
                "href": "plot-radial.html",
                "label": "Radial charts"
            },
            {
                "href": "plot-styling.html",
                "label": "Styling"
            },
            {
                "href": "plot-interaction.html",
                "label": "Interaction"
            },
            {
                "href": "plot-view-state.html",
                "label": "View state"
            },
            {
                "href": "plot-performance.html",
                "label": "Performance"
            }
        ]
    }
];
    const currentPage = window.location.pathname.split('/').pop() || 'index.html';

    function createLink(item) {
        const link = document.createElement('a');
        link.setAttribute('href', item.href);
        link.textContent = item.label;
        if (item.href === currentPage) link.setAttribute('aria-current', 'page');
        return link;
    }

    const sidebar = document.querySelector('[data-site-sidebar]');
    if (sidebar) {
        sidebar.replaceChildren(...sections.map(section => {
            const nav = document.createElement('nav');
            nav.setAttribute('aria-label', section.title);
            const label = document.createElement('div');
            label.className = 'nav-label';
            label.textContent = section.title;
            nav.append(label, ...section.items.map(createLink));
            return nav;
        }));
    }

    const mobile = document.querySelector('[data-site-mobile-nav]');
    if (mobile) {
        const details = document.createElement('details');
        details.className = 'mobile-nav';
        const summary = document.createElement('summary');
        summary.textContent = 'Documentation navigation';
        const nav = document.createElement('nav');
        nav.setAttribute('aria-label', 'Mobile navigation');
        nav.append(...sections.map(section => {
            const group = document.createElement('div');
            group.className = 'mobile-nav-group';
            const label = document.createElement('strong');
            label.textContent = section.title;
            group.append(label, ...section.items.map(createLink));
            return group;
        }));
        details.append(summary, nav);
        mobile.replaceChildren(details);
    }
})();
