#!/usr/bin/env bash
set -euo pipefail

# Usage: ./dump-src.sh [dir]
# Default directory is "src"
ROOT="${1:-src}"

# Extensions to ignore (path only)
IGNORE_EXTS="jpg|jpeg|png|pdf|css"

find "$ROOT" -type f -print0 \
| sort -z \
| while IFS= read -r -d '' file; do
  ext="${file##*.}"
  if [[ "$ext" =~ ^($IGNORE_EXTS)$ ]]; then
    # Ignore list → print path only
    echo "$file"
    echo
  else
    # Everything else → print path + content
    echo "$file"
    cat "$file"
    echo
  fi
done

