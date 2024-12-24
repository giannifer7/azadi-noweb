#!/bin/bash

# Directory containing the files (default is the current directory)
DIRECTORY=${1:-aza3}

# Iterate through files with .md.txt extension in the specified directory
for file in "$DIRECTORY"/*.md.txt; do
  # Check if the file exists (in case there are no matching files)
  if [[ -f "$file" ]]; then
    # Generate the new file name
    new_name="${file%.txt}"
    # Rename the file
    mv "$file" "$new_name"
    echo "Renamed: $file -> $new_name"
  fi
done

echo "Renaming complete!"
