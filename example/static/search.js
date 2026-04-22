import Fuse from "https://cdnjs.cloudflare.com/ajax/libs/fuse.js/7.0.0/fuse.basic.min.mjs";

function buildSnippet(text, start, len) {
    const end = start + len;
    const snippetStart = Math.max(0, start - 40);
    const snippetEnd = Math.min(text.length, end + 40);
    const prefix = snippetStart > 0 ? '...' : '';
    const suffix = snippetEnd < text.length ? '...' : '';
    const before = escapeHtml(text.substring(snippetStart, start));
    const matched = escapeHtml(text.substring(start, end));
    const after = escapeHtml(text.substring(end, snippetEnd));
    return `${prefix}${before}<mark>${matched}</mark>${after}${suffix}`;
}

function getMatchSnippets(result, searchPattern, maxCount) {
    if (!result.matches?.length) return [];
    const match = result.matches.find(m => m.key === "html")
                  || result.matches.find(m => m.key === "description");
    if (!match) return [];
    const text = match.value;
    const textLower = text.toLowerCase();

    const candidates = [searchPattern, ...searchPattern.split(/\s+/)].filter(t => t.length > 2);
    candidates.sort((a, b) => b.length - a.length);

    const snippets = [];
    const usedRanges = [];

    for (const term of candidates) {
        const termLower = term.toLowerCase();
        let searchFrom = 0;
        while (snippets.length < maxCount) {
            const idx = textLower.indexOf(termLower, searchFrom);
            if (idx === -1) break;
            searchFrom = idx + termLower.length;
            const overlaps = usedRanges.some(([s, e]) =>
                idx < e + 40 && idx + termLower.length > s - 40
            );
            if (overlaps) continue;
            usedRanges.push([idx, idx + termLower.length]);
            snippets.push(buildSnippet(text, idx, termLower.length));
        }
        if (snippets.length >= maxCount) break;
    }

    // Fallback to longest Fuse.js index ranges
    if (snippets.length === 0 && match.indices?.length) {
        const sorted = [...match.indices].sort((a, b) => (b[1] - b[0]) - (a[1] - a[0]));
        for (const idx of sorted) {
            if (snippets.length >= maxCount) break;
            snippets.push(buildSnippet(text, idx[0], idx[1] - idx[0] + 1));
        }
    }

    return snippets;
}

function escapeHtml(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

(async () => {
    const searchInput = document.getElementById("marmite-search-input");
    const showMatches = searchInput?.dataset.showMatches === "true";
    const matchCount = parseInt(searchInput?.dataset.matchCount, 10) || 3;

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
                            for (const snippet of getMatchSnippets(result, searchPattern, matchCount)) {
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
    if (document.body.classList.contains('show')) {
        searchInput.focus();
    }
};

document.getElementById("search-toggle").addEventListener("click", toggleSearchBar);
document.getElementById("search-close").addEventListener("click", toggleSearchBar);
document.getElementById("overlay-close").addEventListener("click", toggleSearchBar);

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
