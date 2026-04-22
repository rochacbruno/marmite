import Fuse from "https://cdnjs.cloudflare.com/ajax/libs/fuse.js/7.0.0/fuse.basic.min.mjs";

function getMatchSnippet(result, searchPattern) {
    if (!result.matches?.length) return null;
    const match = result.matches.find(m => m.key === "html")
                  || result.matches.find(m => m.key === "description");
    if (!match) return null;
    const text = match.value;
    const textLower = text.toLowerCase();

    // Try full query first, then individual terms, pick the longest direct hit
    const candidates = [searchPattern, ...searchPattern.split(/\s+/)].filter(t => t.length > 2);
    let bestStart = -1;
    let bestLen = 0;
    for (const term of candidates) {
        const idx = textLower.indexOf(term.toLowerCase());
        if (idx !== -1 && term.length > bestLen) {
            bestStart = idx;
            bestLen = term.length;
        }
    }

    // Fallback to longest Fuse.js index range
    if (bestStart === -1 && match.indices?.length) {
        let best = match.indices[0];
        for (const idx of match.indices) {
            if ((idx[1] - idx[0]) > (best[1] - best[0])) best = idx;
        }
        bestStart = best[0];
        bestLen = best[1] - best[0] + 1;
    }

    if (bestStart === -1) return null;

    const bestEnd = bestStart + bestLen;
    const snippetStart = Math.max(0, bestStart - 40);
    const snippetEnd = Math.min(text.length, bestEnd + 40);
    const prefix = snippetStart > 0 ? '...' : '';
    const suffix = snippetEnd < text.length ? '...' : '';
    const before = escapeHtml(text.substring(snippetStart, bestStart));
    const matched = escapeHtml(text.substring(bestStart, bestEnd));
    const after = escapeHtml(text.substring(bestEnd, snippetEnd));
    return `${prefix}${before}<mark>${matched}</mark>${after}${suffix}`;
}

function escapeHtml(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

(async () => {
    const searchInput = document.getElementById("marmite-search-input");
    const showMatches = searchInput?.dataset.showMatches === "true";

    const fuseOptions = {
        threshold: 0.25,
        findAllMatches: true,
        shouldSort: true,
        minMatchCharLength: 3,
        ignoreLocation: true,
        includeMatches: showMatches,
        keys: ["title", "description", "tags", "html"]
    };

    try {
        const response = await fetch('./static/search_index.json');
        const data = await response.json();
        const fuse = new Fuse(data, fuseOptions);
        searchInput.addEventListener("input", (event) => {
            event.preventDefault();

            // Clear previous results
            const rootElement = document.querySelector(".marmite-search-bar-result");
            rootElement.setAttribute("style", "display: none;");

            const resultsElement = document.querySelector("#marmite-search-bar-result");
            resultsElement.innerHTML = "";

            // Search for results
            const searchPattern = event.target.value;
            if (searchPattern?.length > 2) {
                const results = fuse.search(searchPattern);
                if(results?.length > 0) {
                    // Build the results list, limiting here to 10 items
                    results.slice(0, 10).forEach((result) => {
                        const elementList = document.createElement("li");
                        const resultElement = document.createElement("a");
                        resultElement.href = `${result.item.slug}.html`;
                        resultElement.innerText = result.item.title;
                        elementList.appendChild(resultElement);
                        if (showMatches) {
                            const snippet = getMatchSnippet(result, searchPattern);
                            if (snippet) {
                                const snippetEl = document.createElement("p");
                                snippetEl.className = "search-match-snippet";
                                snippetEl.innerHTML = snippet;
                                elementList.appendChild(snippetEl);
                            }
                        }
                        resultsElement.appendChild(elementList);
                    });
                } else {
                    const elementList = document.createElement("li");
                    const resultElement = document.createElement("span");
                    resultElement.textContent = "No results found";
                    elementList.appendChild(resultElement);
                    resultsElement.appendChild(elementList);
                }
                rootElement.setAttribute("style", "display: block;");
            }
        });
    } catch (error) {
        console.error('Error loading search data:', error);
    }
})();

const toggleSearchBar = () => {
    document.body.classList.toggle('show');
    document.getElementById("marmite-search-input").value = "";
    document.getElementById("marmite-search-bar-result").innerHTML = "";
    // Focus the search input if the search bar is shown
    const searchInput = document.getElementById("marmite-search-input");
    if (searchInput && document.body.classList.contains('show')) {
        searchInput.focus();
    }
};

document.getElementById("search-toggle")?.addEventListener("click", toggleSearchBar);
document.getElementById("search-close")?.addEventListener("click", toggleSearchBar);
document.getElementById("overlay-close")?.addEventListener("click", toggleSearchBar);

// Event listener for keyboard shortcuts
document.addEventListener("keydown", (event) => {
    const searchBarIsVisible = document.body.classList.contains('show');

    // Show on 'Ctrl + Shift + F' key
    if (event.ctrlKey && event.shiftKey && event.key === 'F') {
        toggleSearchBar();
    }

    // Hide on 'Escape' key
    if (event.key === 'Escape' && searchBarIsVisible) {
        toggleSearchBar();
    }
});
