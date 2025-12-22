/*
  Clean Marmite Theme JavaScript

  This file contains basic interactivity for the theme.
  You can extend it with your own custom functionality.
*/

// Theme switcher - light/dark
const themeSwitcher = {
    // Config
    _scheme: "auto",
    toggleButton: document.querySelectorAll(".theme-toggle"),
    rootAttribute: "data-theme",
    localStorageKey: "picoPreferredColorScheme",

    // Init
    init() {
        this.scheme = this.schemeFromLocalStorage;
        this.initToggle();
        this.updateIcon();
    },

    // Get color scheme from local storage
    get schemeFromLocalStorage() {
        return window.localStorage?.getItem(this.localStorageKey);
    },

    // Preferred color scheme
    get preferredColorScheme() {
        return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    },

    // Init toggle
    initToggle() {
        // for each toggle button add event listener
        this.toggleButton.forEach((button) => {
            button.addEventListener(
                "click",
                (event) => {
                    event.preventDefault();
                    // Toggle scheme
                    this.scheme = this.scheme === "dark" ? "light" : "dark";
                    this.updateIcon();
                },
                false
            );
        });
    },

    // Set scheme
    set scheme(scheme) {
        if (scheme == "auto") {
            this._scheme = this.preferredColorScheme;
        } else if (scheme == "dark" || scheme == "light") {
            this._scheme = scheme;
        }
        this.applyScheme();
        this.schemeToLocalStorage();
    },

    // Get scheme
    get scheme() {
        return this._scheme;
    },

    // Apply scheme
    applyScheme() {
        document.querySelector("html")?.setAttribute(this.rootAttribute, this.scheme);
        const githubTheme = this.scheme === "dark" ? "-dark" : "";
        document.querySelector("#highlightjs-theme")?.setAttribute("href", `https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.10.0/styles/github${githubTheme}.min.css`);
    },

    // Store scheme to local storage
    schemeToLocalStorage() {
        window.localStorage?.setItem(this.localStorageKey, this.scheme);
    },

    // Update icon based on the current scheme
    updateIcon() {
        // for each toggle button update icon
        this.toggleButton.forEach((button) => {
            if (this.scheme === "dark") {
                button.innerHTML = "&#9788;"; // Sun icon for light mode
                button.title = "light mode";
            } else {
                button.innerHTML = "&#9789;"; // Moon icon for dark mode
                button.title = "dark mode";
            }
        });
    },
};

// Init theme switcher
themeSwitcher.init();

// Colorscheme switcher
function colorschemeSwitcher() {
    const colorschemes = [
        'catppuccin',
        'clean',
        'dracula',
        'github',
        'gruvbox',
        'iceberg',
        'minimal',
        'minimal_wb',
        'monokai',
        'nord',
        'one',
        'solarized',
        'typewriter'
    ];

    const colorschemeDropdown = document.querySelectorAll('.colorscheme-toggle');

    colorschemeDropdown.forEach((dropdown) => {

        dropdown.addEventListener('change', function () {
            const colorscheme = this.value;
            const colorschemeLink = document.querySelector('#colorscheme-link');
            if (colorscheme === 'default') {
                if (colorschemeLink) {
                    colorschemeLink.remove();
                }

                localStorage.removeItem('marmitePreferredColorScheme');
                return;
            }
            if (colorschemeLink) {
                colorschemeLink.href = `static/colorschemes/${colorscheme}.css`;
            } else {
                const link = document.createElement('link');
                link.id = 'colorscheme-link';
                link.rel = 'stylesheet';
                link.href = `static/colorschemes/${colorscheme}.css`;
                document.head.appendChild(link);
            }
            localStorage.setItem('marmitePreferredColorScheme', colorscheme);

            colorschemeDropdown.forEach((dropdown) => {
                dropdown.value = colorscheme;
            });
        });

        colorschemes.forEach((colorscheme) => {
            const option = document.createElement('option');
            option.value = colorscheme;
            option.textContent = colorscheme;
            dropdown.appendChild(option);
        });

        const colorscheme = localStorage.getItem('marmitePreferredColorScheme');
        if (colorscheme) {
            dropdown.value = colorscheme;
            dropdown.dispatchEvent(new Event('change'));
        }
    });
}

// Add event listener for system theme changes
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', e => {
    themeSwitcher.scheme = e.matches ? 'dark' : 'light';
});

document.addEventListener('DOMContentLoaded', function() {
    // Search functionality
    initializeSearch();
    
    // Mobile menu toggle (if you add a mobile menu)
    initializeMobileMenu();
    
    // Smooth scrolling for anchor links
    initializeSmoothScroll();
    
    // External link handling
    initializeExternalLinks();
});

/**
 * Initialize search overlay functionality
 */
function initializeSearch() {
    const searchToggle = document.getElementById('search-toggle');
    const searchOverlay = document.getElementById('search-overlay');
    const searchClose = document.getElementById('search-close');

    if (!searchToggle || !searchOverlay) {
        return; // Search not enabled
    }

    // Show search overlay
    searchToggle.addEventListener('click', function(e) {
        e.preventDefault();
        searchOverlay.style.display = 'flex';
        const searchInput = document.getElementById('search-input');
        if (searchInput) searchInput.focus();
    });

    // Hide search overlay
    function hideSearch() {
        searchOverlay.style.display = 'none';
        const searchInput = document.getElementById('search-input');
        if (searchInput) searchInput.value = '';
        const searchResults = document.getElementById('search-results');
        if (searchResults) searchResults.innerHTML = '';
    }
    
    if (searchClose) {
        searchClose.addEventListener('click', hideSearch);
    }
    
    // Hide on overlay click
    searchOverlay.addEventListener('click', function(e) {
        if (e.target === searchOverlay) {
            hideSearch();
        }
    });
    
    // Hide on Escape key
    document.addEventListener('keydown', function(e) {
        if (e.key === 'Escape' && searchOverlay.style.display === 'flex') {
            hideSearch();
        }
    });
    
    // Show search with Ctrl+Shift+F (or Cmd+Shift+F on Mac)
    document.addEventListener('keydown', function(e) {
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'F') {
            e.preventDefault();
            searchOverlay.style.display = 'flex';
            const searchInput = document.getElementById('search-input');
            if (searchInput) searchInput.focus();
        }
    });
}

/**
 * Initialize mobile menu functionality
 */
function initializeMobileMenu() {
    const menuToggle = document.getElementById('menu-toggle');
    const menu = document.querySelector('.site-nav ul');
    
    if (!menuToggle || !menu) {
        return;
    }
    
    menuToggle.addEventListener('click', function() {
        menu.classList.toggle('mobile-menu-open');
    });
}

/**
 * Initialize smooth scrolling for anchor links
 */
function initializeSmoothScroll() {
    // Smooth scrolling for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });
}

/**
 * Initialize external link handling
 */
function initializeExternalLinks() {
    // Open external links in new tab
    document.querySelectorAll('a[href^="http"]').forEach(link => {
        if (!link.hostname || link.hostname !== window.location.hostname) {
            link.target = '_blank';
            link.rel = 'noopener noreferrer';
            
            // Add external link indicator
            if (!link.querySelector('.external-indicator')) {
                const indicator = document.createElement('span');
                indicator.className = 'external-indicator';
                indicator.innerHTML = ' ↗';
                indicator.setAttribute('aria-hidden', 'true');
                link.appendChild(indicator);
            }
        }
    });
}

/**
 * Utility function to debounce function calls
 */
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

/**
 * Utility function to throttle function calls
 */
function throttle(func, limit) {
    let inThrottle;
    return function() {
        const args = arguments;
        const context = this;
        if (!inThrottle) {
            func.apply(context, args);
            inThrottle = true;
            setTimeout(() => inThrottle = false, limit);
        }
    };
}

