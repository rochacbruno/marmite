/* Customize me */

// Function to change Giscus theme
function changeGiscusTheme(newTheme) {
  function sendMessage(message) {
    const iframe = document.querySelector('iframe.giscus-frame');
    if (!iframe) return;
    iframe.contentWindow.postMessage({ giscus: message }, 'https://giscus.app');
  }
  
  sendMessage({
    setConfig: {
      theme: newTheme
    }
  });
}

// Add event listener for system theme changes
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', e => {
  // Update theme switcher scheme
  themeSwitcher.scheme = e.matches ? 'dark' : 'light';
  
  // Update giscus theme
  changeGiscusTheme(e.matches ? 'dark' : 'light');
});

// Set initial giscus theme based on current theme when page loads
document.addEventListener("DOMContentLoaded", function() {
  // Check if giscus is present
  if (document.querySelector('iframe.giscus-frame')) {
    const isDarkMode = themeSwitcher.scheme === 'dark';
    changeGiscusTheme(isDarkMode ? 'dark' : 'light');
  }
});
