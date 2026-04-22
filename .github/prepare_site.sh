#!/bin/bash
set -e

rm -rf marmitesite
cp -R example marmitesite
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