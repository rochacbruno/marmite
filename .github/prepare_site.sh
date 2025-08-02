#!/bin/bash
set -e

rm -rf marmitesite
cp -R example marmitesite
# template customization
sed -i '/block head/a <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.10.0/styles/github.min.css" id="highlightjs-theme" />' marmitesite/templates/base.html
sed -i '/block tail/a <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.10.0/highlight.min.js"><\/script>\n<script>\n  hljs.highlightAll();\n<\/script>' marmitesite/templates/base.html
sed -i 's/hljs\.highlightAll();/\/\/ hljs.highlightAll();/' marmitesite/templates/content.html
sed -i '/<link rel="stylesheet" href="https:\/\/cdnjs.cloudflare.com\/ajax\/libs\/highlight.js\/11.10.0\/styles\/github.min.css" id="highlightjs-theme" \/>/d' marmitesite/templates/content.html
# /template customization
echo "---" >> marmitesite/content/docs.md
echo "pinned: true" >> marmitesite/content/docs.md
CURRENT_DATE=$(date +%Y-%m-%d-%H-%M-%S)
echo "date: $CURRENT_DATE" >> marmitesite/content/docs.md
echo "tags: docs" >> marmitesite/content/docs.md
echo "---" >> marmitesite/content/docs.md
cat marmitesite/ai/llms.txt >> marmitesite/content/docs.md
rm marmitesite/marmite.yaml
cp .github/marmite.yaml marmitesite/marmite.yaml
VERSION=$(marmite --version | awk '{print $2}')
sed -i "s/^name: Marmite$/name: Marmite $VERSION/" marmitesite/marmite.yaml
cp .github/_hero.md marmitesite/content/_hero.md