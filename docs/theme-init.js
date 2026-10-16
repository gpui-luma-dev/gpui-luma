// Load synchronously before CSS to apply the saved theme before the first paint.
// Without a saved choice, site.css follows the browser's color-scheme preference.
(() => {
    try {
        const theme = localStorage.getItem('luma-docs-theme');
        if (theme === 'light' || theme === 'dark') {
            document.documentElement.dataset.theme = theme;
        }
    } catch {
        // Storage may be unavailable; keep following the browser preference.
    }
})();
