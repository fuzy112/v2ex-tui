#!/bin/sh
# Extract node list from V2EX planes page
# Usage: ./extract_nodes.sh

curl -s "https://www.v2ex.com/planes" | \
  grep -o 'href="/go/[^"]*"' | \
  sed 's/href="\/go\///g;s/"//g' | \
  sort | \
  uniq