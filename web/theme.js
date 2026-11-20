// Runs before the stylesheets on all three pages. Only an explicit choice is stored.
(() => {
  const key = 'pension-theme';
  const scheme = document.querySelector('meta[name="color-scheme"]');
  const [canvas, darkCanvas] = document.querySelectorAll('meta[name="theme-color"]');
  const colors = { light: canvas.content, dark: darkCanvas.content };
  darkCanvas.remove();
  canvas.removeAttribute('media');
  if (!window.CSS?.supports('color', 'light-dark(white, black)')) {
    scheme.content = 'light';
    return;
  }

  const system = matchMedia('(prefers-color-scheme: dark)');

  const valid = value => value === 'light' || value === 'dark';
  let storage, choice, button;
  try {
    storage = window.localStorage;
    choice = storage.getItem(key);
  } catch { /* Storage may be blocked. */ }

  function apply() {
    const theme = valid(choice) ? choice : system.matches ? 'dark' : 'light';
    document.documentElement.dataset.theme = theme;
    scheme.content = theme;
    canvas.content = colors[theme];
    if (button) {
      button.textContent = theme === 'dark' ? '☀️' : '🌙';
      button.setAttribute('aria-label', theme === 'dark' ? '切换到浅色模式' : '切换到深色模式');
    }
  }
  apply();
  system.addEventListener('change', apply);
  window.addEventListener('storage', event => {
    if (event.key === key && valid(event.newValue) && event.storageArea === storage) {
      choice = event.newValue;
      apply();
    }
  });
  document.addEventListener('DOMContentLoaded', () => {
    button = document.getElementById('theme-toggle');
    button.addEventListener('click', () => {
      choice = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
      apply();
      try { storage?.setItem(key, choice); } catch { /* Keep this page's choice. */ }
    });
    apply();
    button.hidden = false;
  });
})();
