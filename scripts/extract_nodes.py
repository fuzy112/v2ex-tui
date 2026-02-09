#!/usr/bin/env python3
"""
Admin script: Extract V2EX nodes from website

This script scrapes the V2EX planes page to extract all available nodes
and generates various output formats for the v2ex-tui project.

Usage:
    python scripts/extract_nodes.py

Output:
    - nodes.json: Complete node list
    - Console output: Rust code snippets
"""

import re
import sys
import subprocess
import json
from pathlib import Path

def extract_nodes():
    """Extract node list from V2EX planes page"""
    try:
        # Fetch the planes page
        import urllib.request
        import urllib.error
        url = "https://www.v2ex.com/planes"
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        response = urllib.request.urlopen(req)
        html = response.read().decode('utf-8')
    except ImportError:
        # Fallback to curl if urllib not available
        result = subprocess.run(
            ['curl', '-s', 'https://www.v2ex.com/planes'],
            capture_output=True,
            text=True
        )
        html = result.stdout
    
    # Extract nodes with pattern: href="/go/NODE_NAME" class="item_node">NODE_TITLE</a>
    # Use regex to capture both node name and title
    pattern = r'href="/go/([^"]+)"[^>]*>([^<]+)</a>'
    matches = re.findall(pattern, html)
    
    # Filter for item_node class (more specific)
    pattern2 = r'href="/go/([^"]+)" class="item_node">([^<]+)</a>'
    matches2 = re.findall(pattern2, html)
    
    # Use the more specific matches if found
    if matches2:
        nodes = matches2
    else:
        nodes = matches
    
    # Remove duplicates while preserving order
    seen = set()
    unique_nodes = []
    for node_name, node_title in nodes:
        if node_name not in seen:
            seen.add(node_name)
            unique_nodes.append((node_name, node_title))
    
    return unique_nodes



def main():
    nodes = extract_nodes()
    
    print(f"Found {len(nodes)} nodes")
    
    # Show summary of favorite nodes
    favorite_nodes = nodes[:9]
    print("\n\nFavorite nodes (first 9):")
    for i, (name, title) in enumerate(favorite_nodes, 1):
        print(f"{i}. {name:20} - {title}")
    
    # Save to file in scripts/ directory
    script_dir = Path(__file__).parent
    output_file = script_dir / 'nodes.json'
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(nodes, f, ensure_ascii=False, indent=2)
    print(f"\nNode list saved to {output_file}")

if __name__ == "__main__":
    main()