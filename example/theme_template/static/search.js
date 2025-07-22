import Fuse from "https://cdnjs.cloudflare.com/ajax/libs/fuse.js/7.0.0/fuse.basic.min.mjs";
(async () => {
    const fuseOptions = {
        threshold: 0.25,
        findAllMatches: true,
        shouldSort: true,
        minMatchCharLength: 3,
        ignoreLocation: true,
        keys: ["title", "description", "tags", "html"]
    };

    try {
        const response = await fetch('./static/search_index.json');
        const data = await response.json();
        const fuse = new Fuse(data, fuseOptions);
        document.getElementById("marmite-search-input").addEventListener("input", (event) => {
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
