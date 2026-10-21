const status = document.querySelector('#status');
const params = new URLSearchParams(location.search);
const systemTheme = matchMedia('(prefers-color-scheme: dark)');
let dark = params.get('theme') === 'dark' || (!params.has('theme') && systemTheme.matches);
let applyTheme;
const startupTimeout = setTimeout(() => {
    fail('The interactive preview is taking longer than expected. Try a browser with WebGPU or WebGL2 support.');
}, 30000);
function updateTheme(value) {
    dark = value;
    document.documentElement.style.colorScheme = dark ? 'dark' : 'light';
    applyTheme?.(dark);
}
function fail(message) {
    clearTimeout(startupTimeout);
    status.hidden = false;
    status.textContent = `${message} The documentation and code examples remain available.`;
}
window.addEventListener('message', event => {
    if (event.origin === location.origin && event.source === parent &&
        event.data?.type === 'luma-theme' && typeof event.data.dark === 'boolean') {
        updateTheme(event.data.dark);
    }
});
systemTheme.addEventListener('change', event => { if (parent === window) updateTheme(event.matches); });
window.addEventListener('error', () => fail('The interactive example could not start.'));
window.addEventListener('unhandledrejection', () => fail('The interactive example could not start.'));
window.addEventListener('luma-ready', () => {
    clearTimeout(startupTimeout);
    status.hidden = true;
    if (parent !== window) parent.postMessage({ type: 'luma-ready' }, location.origin);
});
updateTheme(dark);
try {
    if (location.protocol === 'file:') throw new Error('Serve the documentation over HTTP to run examples.');
    const response = await fetch('./pkg/manifest.json', { cache: 'no-store' });
    if (!response.ok) throw new Error('The example runtime has not been built or published.');
    const manifest = await response.json();
    const runtime = await import(`./pkg/${manifest.version}/luma_docs_embed.js`);
    await runtime.default();
    runtime.start(params.get('id') || 'button-basic');
    applyTheme = runtime.set_theme;
    updateTheme(dark);
} catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    fail(`Interactive preview unavailable: ${detail}`);
    console.error(error);
}
