// Shared documentation chrome and interactions. Works on file:// and GitHub Pages.
(() => {
    const header = document.querySelector('[data-site-header]');
    if (header) header.innerHTML = `<a class="brand" href="index.html" aria-label="Luma SDK documentation"><i class="mark" aria-hidden="true"></i>Luma<span>Documentation</span></a>
<div class="header-right"><a href="https://github.com/gpui-luma-dev/gpui-luma">GitHub ↗</a><button class="theme" id="theme" aria-pressed="false" aria-label="Switch to dark theme"><svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><circle cx="10" cy="10" r="7"/><path d="M10 3a7 7 0 0 1 0 14Z" fill="currentColor" stroke="none"/></svg><span id="theme-label">Dark mode</span></button></div>`;

    const footer = document.querySelector('[data-site-footer]');
    if (footer) {
        const context = document.createElement('span');
        context.textContent = `Luma SDK · ${document.title.split(' · ')[0]}`;
        const attribution = document.createElement('span');
        attribution.textContent = 'Documentation created by Codex';
        footer.replaceChildren(context, attribution);
    }

const root = document.documentElement;
const theme = document.getElementById('theme');
const systemTheme = window.matchMedia('(prefers-color-scheme: dark)');
function isDarkTheme() {
    return root.dataset.theme === 'dark' || (!root.dataset.theme && systemTheme.matches);
}
function syncTheme() {
    const dark = isDarkTheme();
    theme.setAttribute('aria-pressed', String(dark));
    theme.setAttribute('aria-label', dark ? 'Switch to light theme' : 'Switch to dark theme');
    document.getElementById('theme-label').textContent = dark ? 'Light mode' : 'Dark mode';
}
syncTheme();
systemTheme.addEventListener('change', syncTheme);
theme.addEventListener('click', () => {
    const next = isDarkTheme() ? 'light' : 'dark';
    root.dataset.theme = next;
    try { localStorage.setItem('luma-docs-theme', next); } catch {}
    syncTheme();
});
// Highlight only explicitly marked code blocks; copying still reads plain text.
if (window.Prism) window.Prism.highlightAll();
document.querySelectorAll('.copy').forEach(button => {
    button.addEventListener('click', async () => {
        const code = button.closest('.codebox').querySelector('code');
        try {
            await navigator.clipboard.writeText(code.textContent);
            button.textContent = 'Copied';
        } catch {
            const selection = window.getSelection();
            const range = document.createRange();
            range.selectNodeContents(code);
            selection.removeAllRanges();
            selection.addRange(range);
            button.textContent = 'Selected — press copy';
        }
        setTimeout(() => { button.textContent = 'Copy'; }, 2200);
    });
});
})();
