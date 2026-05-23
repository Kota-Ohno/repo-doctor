#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    echo "Usage: $0 <project-name> [destination-directory]" >&2
    exit 2
fi

project_name="$1"
crate_name="${project_name//-/_}"
destination="${2:-$HOME/src/$project_name}"
template_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ ! "$project_name" =~ ^[a-z][a-z0-9_-]*$ ]]; then
    echo "Project name must start with a lowercase letter and contain only lowercase letters, numbers, hyphens, or underscores." >&2
    exit 2
fi

if [ -e "$destination" ]; then
    echo "Destination already exists: $destination" >&2
    exit 1
fi

mkdir -p "$(dirname "$destination")"
rsync -a \
    --exclude .git \
    --exclude target \
    "$template_dir/" "$destination/"

while IFS= read -r -d '' file; do
    sed -i "s/repo-doctor/$project_name/g" "$file"
    sed -i "s/repo_doctor/$crate_name/g" "$file"
done < <(find "$destination" -type f -not -path '*/.git/*' -print0)

(
    cd "$destination"
    git init
    cargo generate-lockfile
)

echo "Created $destination"
