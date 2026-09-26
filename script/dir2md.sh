#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# dir2md.sh
#
# Directory -> LLM-friendly Markdown converter
# ==============================================================================

# ------------------------------------------------------------------------------
# Default settings
# ------------------------------------------------------------------------------

INPUT_DIR=""
OUTPUT_MD=""

MAX_SIZE=500000

INCLUDE_BINARY=false
USE_GITIGNORE=false
INCLUDE_HIDDEN=false

TREE_ONLY=false
NO_TREE=false
QUIET=false

IMPORTANT_PATTERNS=(
    README* LICENSE* Makefile Dockerfile
    "*.c" "*.cc" "*.cpp" "*.h" "*.hh" "*.hpp"
    "*.rs" "*.py" "*.pyi" "*.js" "*.jsx" "*.ts" "*.tsx"
    "*.pl"
    "*.java" "*.kt" "*.kts" "*.swift" "*.go"
    "*.sh" "*.bash" "*.zsh"
    "*.html" "*.css" "*.scss" "*.json" "*.jsonl"
    "*.yaml" "*.yml" "*.toml" "*.xml" "*.md" "*.txt" "*.csv" "*.sql"
    "*.gradle" "*.properties"
    "requirements*.txt" "package*.json" "Cargo.toml" "Cargo.lock"
    "go.mod" "go.sum" "pyproject.toml" "setup.py" "build.gradle" "pom.xml"
)

IGNORE_DIRS=(
    ".git" ".hg" ".svn" "node_modules" "target" "dist" "build"
    ".venv" "venv" "__pycache__" ".pytest_cache" ".mypy_cache"
    ".idea" ".vscode" ".DS_Store"
)

EXCLUDE_PATTERNS=()

# ------------------------------------------------------------------------------
# Usage
# ------------------------------------------------------------------------------

usage() {
    cat <<EOF
Usage:
  $0 <directory> [options]

Options:
  -o, --output FILE       Output markdown file
  -m, --max-size BYTES    Maximum file size to include (default: ${MAX_SIZE})
      --no-size-limit     Do not limit file size
      --binary            Include binary files
      --no-binary         Exclude binary files (default)
      --gitignore         Respect .gitignore
      --hidden            Include hidden files/directories
      --all               Include all files instead of source-file patterns
      --tree-only         Output only directory/file tree
      --no-tree           Do not output directory/file tree
      --exclude PATTERN   Exclude files/directories matching PATTERN
  -q, --quiet             Suppress progress messages
  -h, --help              Show this help
EOF
}

# ------------------------------------------------------------------------------
# Argument parsing
# ------------------------------------------------------------------------------

while [[ $# -gt 0 ]]; do
    case "$1" in
        -o|--output)
            [[ $# -ge 2 ]] || { echo "Error: $1 requires an argument." >&2; exit 1; }
            OUTPUT_MD="$2"; shift 2 ;;
        -m|--max-size)
            [[ $# -ge 2 ]] || { echo "Error: $1 requires an argument." >&2; exit 1; }
            MAX_SIZE="$2"
            if ! [[ "$MAX_SIZE" =~ ^[0-9]+$ ]]; then
                echo "Error: max size must be an integer." >&2; exit 1
            fi
            shift 2 ;;
        --no-size-limit) MAX_SIZE=0; shift ;;
        --binary)        INCLUDE_BINARY=true; shift ;;
        --no-binary)     INCLUDE_BINARY=false; shift ;;
        --gitignore)     USE_GITIGNORE=true; shift ;;
        --hidden)        INCLUDE_HIDDEN=true; shift ;;
        --all)           IMPORTANT_PATTERNS=("*"); shift ;;
        --tree-only)     TREE_ONLY=true; shift ;;
        --no-tree)       NO_TREE=true; shift ;;
        --exclude)
            [[ $# -ge 2 ]] || { echo "Error: --exclude requires an argument." >&2; exit 1; }
            EXCLUDE_PATTERNS+=("$2"); shift 2 ;;
        -q|--quiet)      QUIET=true; shift ;;
        -h|--help)       usage; exit 0 ;;
        -*)              echo "Error: Unknown option: $1" >&2; usage >&2; exit 1 ;;
        *)
            if [[ -z "$INPUT_DIR" ]]; then
                INPUT_DIR="$1"
            else
                echo "Error: Multiple input directories specified." >&2; exit 1
            fi
            shift ;;
    esac
done

if [[ -z "$INPUT_DIR" ]]; then
    echo "Error: input directory is required." >&2; usage >&2; exit 1
fi
if [[ ! -d "$INPUT_DIR" ]]; then
    echo "Error: directory not found: $INPUT_DIR" >&2; exit 1
fi

INPUT_DIR="$(cd "$INPUT_DIR" || exit 1; pwd)"

if [[ -z "$OUTPUT_MD" ]]; then
    base_name="$(basename "$INPUT_DIR")"
    [[ "$base_name" == "." || -z "$base_name" ]] && base_name="repo_dump"
    OUTPUT_MD="${base_name}.md"
fi

OUTPUT_DIR="$(dirname "$OUTPUT_MD")"
OUTPUT_NAME="$(basename "$OUTPUT_MD")"
if [[ "$OUTPUT_DIR" != /* ]]; then
    OUTPUT_DIR="$(pwd)/$OUTPUT_DIR"
fi
mkdir -p "$OUTPUT_DIR"
OUTPUT_MD="$OUTPUT_DIR/$OUTPUT_NAME"

# ------------------------------------------------------------------------------
# Patterns & Validation Functions
# ------------------------------------------------------------------------------

SECRET_PATTERNS=(
    ".env" ".env.*" "*.pem" "*.key" "*.p12" "*.pfx"
    "*secret*" "*credential*" "*token*" "*password*" "*passwd*" "*auth*"
)

match_secret() {
    local file="$1" base="$(basename "$1")"
    if [[ ${#SECRET_PATTERNS[@]} -gt 0 ]]; then
        for pattern in "${SECRET_PATTERNS[@]}"; do
            if [[ "$base" == $pattern ]]; then return 0; fi
        done
    fi
    return 1
}

match_exclude() {
    local file="$1" relative="${1#"$INPUT_DIR"/}" base="$(basename "$1")"

    if [[ ${#IGNORE_DIRS[@]} -gt 0 ]]; then
        for dir in "${IGNORE_DIRS[@]}"; do
            if [[ "$relative" == "$dir"/* ]] || [[ "$relative" == "$dir" ]]; then return 0; fi
        done
    fi

    if [[ ${#EXCLUDE_PATTERNS[@]} -gt 0 ]]; then
        for pattern in "${EXCLUDE_PATTERNS[@]}"; do
            if [[ "$relative" == $pattern ]] || [[ "$base" == $pattern ]] || [[ "$relative" == $pattern/* ]]; then return 0; fi
        done
    fi

    if [[ "$INCLUDE_HIDDEN" == false ]]; then
        if [[ "$relative" == .* ]] || [[ "$relative" == */.* ]]; then return 0; fi
    fi
    return 1
}

GITIGNORE_PATTERNS=()
if [[ "$USE_GITIGNORE" == true && -f "$INPUT_DIR/.gitignore" ]]; then
    while IFS= read -r line; do
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
        GITIGNORE_PATTERNS+=("$line")
    done < "$INPUT_DIR/.gitignore"
fi

match_gitignore() {
    local file="$1" relative="${1#"$INPUT_DIR"/}" base="$(basename "$1")"
    if [[ ${#GITIGNORE_PATTERNS[@]} -gt 0 ]]; then
        for pattern in "${GITIGNORE_PATTERNS[@]}"; do
            pattern="${pattern#/}"
            if [[ "$relative" == $pattern ]] || [[ "$relative" == $pattern/* ]] || [[ "$base" == $pattern ]]; then return 0; fi
        done
    fi
    return 1
}

is_binary() {
    local file="$1"
    [[ ! -s "$file" ]] && return 1 # 0 byte file is treated as text
    
    if command -v file >/dev/null 2>&1; then
        if file -b --mime-encoding "$file" | grep -q binary; then return 0; fi
        return 1
    fi
    
    if grep -Iq . "$file" 2>/dev/null; then return 1; fi
    return 0
}

file_size() { wc -c < "$1" | tr -d ' '; }

is_important_file() {
    local base="$(basename "$1")"
    if [[ ${#IMPORTANT_PATTERNS[@]} -gt 0 ]]; then
        for pattern in "${IMPORTANT_PATTERNS[@]}"; do
            if [[ "$base" == $pattern ]]; then return 0; fi
        done
    fi
    return 1
}

# ------------------------------------------------------------------------------
# Collect & Sort Files
# ------------------------------------------------------------------------------

FILES=()
while IFS= read -r -d '' file; do
    [[ "$file" == "$OUTPUT_MD" ]] && continue
    match_exclude "$file" && continue
    match_gitignore "$file" && continue
    match_secret "$file" && continue

    if [[ "${IMPORTANT_PATTERNS[*]:-}" != "*" ]]; then
        is_important_file "$file" || continue
    fi

    FILES+=("$file")
done < <(find "$INPUT_DIR" -type f -print0)

# Sort files by relative path safely
if [[ ${#FILES[@]} -gt 0 ]]; then
    IFS=$'\n' sorted_rel=($(
        for f in "${FILES[@]}"; do echo "${f#"$INPUT_DIR"/}"; done | LC_ALL=C sort
    ))
    unset IFS
    FILES=()
    for rel in "${sorted_rel[@]}"; do
        FILES+=("$INPUT_DIR/$rel")
    done
fi

# ------------------------------------------------------------------------------
# Output Generation
# ------------------------------------------------------------------------------

{
    echo "# Repository Dump"
    echo
    echo "Source: \`$INPUT_DIR\`"
    echo "Maximum file size: ${MAX_SIZE:-unlimited} bytes"
    echo
} > "$OUTPUT_MD"


if [[ "$NO_TREE" == false ]]; then
    {
        echo "## File Tree"
        echo
        echo '```text'
        if [[ ${#FILES[@]} -gt 0 ]]; then
            for f in "${FILES[@]}"; do echo "${f#"$INPUT_DIR"/}"; done
        fi
        echo '```'
        echo
    } >> "$OUTPUT_MD"
fi

if [[ "$TREE_ONLY" == true ]]; then
    [[ "$QUIET" == false ]] && echo "Done -> $OUTPUT_MD"
    exit 0
fi

{
    echo "## Files"
    echo
    echo "| # | Path | Size | Type |"
    echo "|---:|---|---:|---|"
} >> "$OUTPUT_MD"

if [[ ${#FILES[@]} -gt 0 ]]; then
    for ((i=0; i<${#FILES[@]}; i++)); do
        file="${FILES[$i]}" relative="${file#"$INPUT_DIR"/}" size="$(file_size "$file")"
        if is_binary "$file"; then type="binary"; else type="text"; fi
        echo "| $((i + 1)) | \`$relative\` | $size bytes | $type |" >> "$OUTPUT_MD"
    done
fi
echo >> "$OUTPUT_MD"

{
    echo "## File Contents"
    echo
} >> "$OUTPUT_MD"

SKIPPED_BINARY=()
SKIPPED_SIZE=()

if [[ ${#FILES[@]} -gt 0 ]]; then
    for file in "${FILES[@]}"; do
        relative="${file#"$INPUT_DIR"/}" size="$(file_size "$file")"

        if is_binary "$file"; then
            if [[ "$INCLUDE_BINARY" == false ]]; then
                SKIPPED_BINARY+=("$relative")
                continue
            fi
            language="binary"
        else
            language="text"
        fi

        if (( MAX_SIZE > 0 && size > MAX_SIZE )); then
            SKIPPED_SIZE+=("$relative ($size bytes)")
            continue
        fi

        extension=""
        [[ "$relative" == *.* ]] && extension="${relative##*.}"
        case "$extension" in
            c|h|cc|cpp|hh|hpp) language="c" ;;
            rs) language="rust" ;;
            py|pyi) language="python" ;;
            js|jsx) language="javascript" ;;
            ts|tsx) language="typescript" ;;
            java) language="java" ;;
            kt|kts) language="kotlin" ;;
            go) language="go" ;;
            swift) language="swift" ;;
            sh|bash) language="bash" ;;
            zsh) language="zsh" ;;
            html) language="html" ;;
            css) language="css" ;;
            json|jsonl) language="json" ;;
            yaml|yml) language="yaml" ;;
            toml) language="toml" ;;
            xml) language="xml" ;;
            sql) language="sql" ;;
            md) language="markdown" ;;
            txt|csv) language="text" ;;
            *) language="text" ;;
        esac

        {
            echo "### File: \`$relative\`"
            echo
            echo "- Size: ${size} bytes"
            echo
            echo "\`\`\`${language}"
            cat "$file"
            echo
            echo "\`\`\`"
            echo
        } >> "$OUTPUT_MD"
    done
fi

if (( ${#SKIPPED_BINARY[@]} > 0 )); then
    {
        echo "## Skipped Binary Files"
        echo
        for file in "${SKIPPED_BINARY[@]}"; do echo "- \`$file\`"; done
        echo
    } >> "$OUTPUT_MD"
fi

if (( ${#SKIPPED_SIZE[@]} > 0 )); then
    {
        echo "## Skipped Large Files"
        echo
        for file in "${SKIPPED_SIZE[@]}"; do echo "- \`$file\`"; done
        echo
    } >> "$OUTPUT_MD"
fi

{
    echo "## Summary"
    echo
    echo "- Files found: ${#FILES[@]}"
    echo "- Binary files skipped: ${#SKIPPED_BINARY[@]}"
    echo "- Large files skipped: ${#SKIPPED_SIZE[@]}"
    echo
    echo "- Output size:"
    echo
    echo '```text'
    wc -c < "$OUTPUT_MD" | tr -d ' '
    echo '```'
} >> "$OUTPUT_MD"

if [[ "$QUIET" == false ]]; then
    echo "Done -> $OUTPUT_MD"
fi