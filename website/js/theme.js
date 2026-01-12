// Theme toggle: respects prefers-color-scheme, persists choice in localStorage
(function () {
    const root = document.documentElement;
    const toggleId = 'theme-toggle';
    const storageKey = 'cadi_theme';

    function applyTheme(theme) {
        if (theme === 'light') {
            root.classList.add('light-theme');
            root.setAttribute('data-theme', 'light');
        } else if (theme === 'dark') {
            root.classList.remove('light-theme');
            root.setAttribute('data-theme', 'dark');
        }
        const btn = document.getElementById(toggleId);
        if (btn) btn.setAttribute('aria-pressed', theme === 'light' ? 'true' : 'false');
    }

    function currentSystemPrefersLight() {
        return window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches;
    }

    // Initialize theme on load
    let stored = null;
    try { stored = localStorage.getItem(storageKey); } catch (e) { /* ignore */ }

    if (stored === 'light' || stored === 'dark') {
        applyTheme(stored);
    } else {
        applyTheme(currentSystemPrefersLight() ? 'light' : 'dark');
    }

    // Attach toggle handler
    function toggle() {
        const isLight = root.classList.contains('light-theme');
        const next = isLight ? 'dark' : 'light';
        try { localStorage.setItem(storageKey, next); } catch (e) { }
        applyTheme(next);
    }

    document.addEventListener('DOMContentLoaded', function () {
        const btn = document.getElementById(toggleId);
        if (btn) {
            btn.addEventListener('click', toggle);
            btn.addEventListener('keyup', (e) => { if (e.key === 'Enter' || e.key === ' ') toggle(); });
            btn.setAttribute('role', 'switch');
            btn.setAttribute('aria-pressed', root.classList.contains('light-theme') ? 'true' : 'false');
        }

        // Listen for system changes if user hasn't explicitly chosen
        window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').addEventListener('change', (e) => {
            let storedValue = null;
            try { storedValue = localStorage.getItem(storageKey); } catch (err) { }
            if (storedValue !== 'light' && storedValue !== 'dark') {
                applyTheme(e.matches ? 'light' : 'dark');
            }
        });
    });
})();