/*
  Clean Marmite Theme JavaScript
  
  This file contains basic interactivity for the theme.
  You can extend it with your own custom functionality.
*/

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
    const searchInput = document.getElementById('search-input');
    
    if (!searchToggle || !searchOverlay) {
        return; // Search not enabled
    }
    
    // Show search overlay
    searchToggle.addEventListener('click', function(e) {
        e.preventDefault();
        searchOverlay.style.display = 'flex';
        searchInput.focus();
    });
    
    // Hide search overlay
    function hideSearch() {
        searchOverlay.style.display = 'none';
        searchInput.value = '';
        document.getElementById('search-results').innerHTML = '';
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
            searchInput.focus();
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

/**
 * Simple theme switcher (light/dark mode)
 * Uncomment if you want to add manual theme switching
 */
/*
function initializeThemeSwitcher() {
    const themeToggle = document.getElementById('theme-toggle');
    const currentTheme = localStorage.getItem('theme') || 'light';
    
    document.documentElement.setAttribute('data-theme', currentTheme);
    
    if (themeToggle) {
        themeToggle.addEventListener('click', function() {
            const currentTheme = document.documentElement.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            
            document.documentElement.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
        });
    }
}
*/