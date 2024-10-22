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
        // staticBase is defined on content.html template
        document.querySelector("#highlightjs-theme")?.setAttribute("href", `${staticBase}/github-${this.scheme}.min.css`);
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


