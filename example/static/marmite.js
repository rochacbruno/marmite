// Theme switcher - light/dark
const themeSwitcher = {
    // Config
    _scheme: "auto",
    toggleButton: document.getElementById("theme-toggle"),
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
        return window.localStorage?.getItem(this.localStorageKey) ?? this._scheme;
    },
    
    // Preferred color scheme
    get preferredColorScheme() {
        return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    },
    
    // Init toggle
    initToggle() {
        this.toggleButton.addEventListener(
            "click",
            (event) => {
                event.preventDefault();
                // Toggle scheme
                this.scheme = this.scheme === "dark" ? "light" : "dark";
                this.updateIcon();
            },
            false
        );
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
        if (this.scheme === "dark") {
            this.toggleButton.innerHTML = "&#9788;"; // Sun icon for light mode
        } else {
            this.toggleButton.innerHTML = "&#9789;"; // Moon icon for dark mode
        }
    },
};

// Init
themeSwitcher.init();

// Menu

const menuToggle = document.getElementById('menu-toggle');
const headerMenu = document.getElementById('header-menu');

menuToggle.addEventListener('click', function () {
    headerMenu.classList.toggle('active');
});


// Selected menu animation
document.addEventListener("DOMContentLoaded", function () {
    const menuItems = document.querySelectorAll('.menu-item');
    const underline = document.querySelector('.underline');

    function setUnderline(item) {
        underline.style.width = `${item.offsetWidth}px`; 
        underline.style.transform = `translateX(${item.offsetLeft}px)`; 
    }

    const activeItem = document.querySelector('.menu-item.active');
    if (activeItem) {
        setUnderline(activeItem); 
    }

    menuItems.forEach(item => {
        item.addEventListener('click', function (event) {
            if (this.classList.contains('active')) {
                return; 
            }

            menuItems.forEach(i => {
                i.classList.remove('active');
            });

            this.classList.add('active');
            
            setUnderline(this);
        });
    });
});

function colorschemeSwitcher() {
    const colorschemes = [
        'colorscheme',
        'catppuccin',
        'dracula',
        'github',
        'gruvbox',
        'iceberg',
        'monokai',
        'nord',
        'one',
        'solarized',
        'typewriter'
    ];

    const colorschemeDropdown = document.querySelector('.colorscheme-toggle');

    colorschemeDropdown.addEventListener('change', function () {
        const colorscheme = this.value;
        const colorschemeLink = document.querySelector('#colorscheme-link');
        if (colorscheme === 'colorscheme') {
            if (colorschemeLink) {
                colorschemeLink.remove();
            }

            localStorage.removeItem('marmitePreferredColorScheme');
            return;
        }
        if (colorschemeLink) {
            colorschemeLink.href = `/static/colorschemes/${colorscheme}.css`;
        } else {
            const link = document.createElement('link');
            link.id = 'colorscheme-link';
            link.rel = 'stylesheet';
            link.href = `/static/colorschemes/${colorscheme}.css`;
            document.head.appendChild(link);
        }
        localStorage.setItem('marmitePreferredColorScheme', colorscheme);
    });

    colorschemes.forEach((colorscheme) => {
        const option = document.createElement('option');
        option.value = colorscheme;
        option.textContent = colorscheme;
        colorschemeDropdown.appendChild(option);
    });

    const colorscheme = localStorage.getItem('marmitePreferredColorScheme');
    if (colorscheme) {
        colorschemeDropdown.value = colorscheme;
        colorschemeDropdown.dispatchEvent(new Event('change'));
    }
}