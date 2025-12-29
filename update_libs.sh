#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="elliottech"
REPO_NAME="lighter-go"
LIBS_DIR="libs"
README_FILE="$LIBS_DIR/README.md"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Change to script directory
cd "$SCRIPT_DIR"

# Check dependencies
command -v curl >/dev/null 2>&1 || { echo -e "${RED}Error: curl is required but not installed.${NC}" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo -e "${RED}Error: jq is required but not installed.${NC}" >&2; exit 1; }
command -v shasum >/dev/null 2>&1 || command -v sha256sum >/dev/null 2>&1 || { echo -e "${RED}Error: shasum or sha256sum is required but not installed.${NC}" >&2; exit 1; }

# Determine which sha256 command to use
if command -v shasum >/dev/null 2>&1; then
    SHA256_CMD="shasum -a 256"
elif command -v sha256sum >/dev/null 2>&1; then
    SHA256_CMD="sha256sum"
fi

# Initialize hash storage (using a simpler approach compatible with all bash versions)
HASH_VERIFICATION_FAILED=0

echo -e "${GREEN}Fetching latest release from ${REPO_OWNER}/${REPO_NAME}...${NC}"

# Fetch latest release
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest")

# Check if we got a valid response
if echo "$LATEST_RELEASE" | jq -e '.tag_name' > /dev/null 2>&1; then
    TAG_NAME=$(echo "$LATEST_RELEASE" | jq -r '.tag_name')
    echo -e "${GREEN}Latest release: ${TAG_NAME}${NC}"
else
    echo -e "${RED}Error: Failed to fetch release information${NC}" >&2
    exit 1
fi

# Create temp directory for downloads
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Function to extract hash from digest (format: "sha256:hash")
extract_digest_hash() {
    local digest="$1"
    echo "$digest" | sed 's/^sha256://'
}

# Function to get expected hash from API digest
get_expected_hash_from_api() {
    local filename="$1"
    local digest=$(echo "$LATEST_RELEASE" | jq -r ".assets[] | select(.name == \"${filename}\") | .digest" | head -1)
    if [[ -n "$digest" && "$digest" != "null" ]]; then
        extract_digest_hash "$digest"
    fi
}

# Function to calculate SHA256 hash
calculate_hash() {
    local file="$1"
    $SHA256_CMD "$file" | awk '{print $1}'
}

# Function to verify file hash
verify_hash() {
    local file="$1"
    local expected_hash="$2"
    
    if [[ -z "$expected_hash" ]]; then
        echo -e "  ${YELLOW}No checksum available for verification${NC}" >&2
        return 0
    fi
    
    local actual_hash=$(calculate_hash "$file")
    
    if [[ "$actual_hash" == "$expected_hash" ]]; then
        echo -e "  ${GREEN}✓ SHA256 verified: ${actual_hash}${NC}" >&2
        return 0
    else
        echo -e "  ${RED}✗ SHA256 mismatch!${NC}" >&2
        echo -e "    Expected: ${expected_hash}" >&2
        echo -e "    Actual:   ${actual_hash}" >&2
        return 1
    fi
}

# Function to download and process a file
download_and_process_file() {
    local asset_name="$1"
    local target_platform="$2"
    local target_arch="$3"
    local target_filename="$4"
    
    local asset_url=$(echo "$LATEST_RELEASE" | jq -r ".assets[] | select(.name == \"${asset_name}\") | .browser_download_url" | head -1)
    local expected_digest=$(echo "$LATEST_RELEASE" | jq -r ".assets[] | select(.name == \"${asset_name}\") | .digest" | head -1)
    
    if [[ -z "$asset_url" || "$asset_url" == "null" ]]; then
        echo -e "  ${YELLOW}Asset ${asset_name} not found in release${NC}" >&2
        return 1
    fi
    
    local target_dir="$LIBS_DIR/$target_platform/$target_arch"
    mkdir -p "$target_dir"
    local target_path="$target_dir/$target_filename"
    
    echo -e "  ${GREEN}Downloading ${asset_name}...${NC}" >&2
    curl -L -s "$asset_url" -o "$TEMP_DIR/$asset_name"
    
    # Verify hash from digest
    if [[ -n "$expected_digest" && "$expected_digest" != "null" ]]; then
        local expected_hash=$(extract_digest_hash "$expected_digest")
        if ! verify_hash "$TEMP_DIR/$asset_name" "$expected_hash" >&2; then
            echo -e "  ${RED}Hash verification failed for ${asset_name}!${NC}" >&2
            HASH_VERIFICATION_FAILED=1
            return 1
        fi
    else
        local calculated_hash=$(calculate_hash "$TEMP_DIR/$asset_name")
        echo -e "  ${YELLOW}SHA256: ${calculated_hash} (no digest in API)${NC}" >&2
    fi
    
    # Copy to target location
    cp "$TEMP_DIR/$asset_name" "$target_path"
    
    # Calculate and return hash for README (stdout only, no echo)
    calculate_hash "$target_path"
}

echo -e "${GREEN}Downloading and processing binaries...${NC}"

# Storage for hashes (using simple variables instead of associative array)
HASH_LINUX_AMD64_SO=""
HASH_LINUX_AMD64_H=""
HASH_LINUX_ARM64_SO=""
HASH_LINUX_ARM64_H=""
HASH_DARWIN_ARM64_DYLIB=""
HASH_DARWIN_ARM64_H=""
HASH_WINDOWS_AMD64_DLL=""
HASH_WINDOWS_AMD64_H=""

# Download Linux amd64
HASH_LINUX_AMD64_SO=$(download_and_process_file "lighter-signer-linux-amd64.so" "linux" "amd64" "liblighter-signer.so" || echo "")
HASH_LINUX_AMD64_H=$(download_and_process_file "lighter-signer-linux-amd64.h" "linux" "amd64" "liblighter-signer.h" || echo "")

# Download Linux arm64
HASH_LINUX_ARM64_SO=$(download_and_process_file "lighter-signer-linux-arm64.so" "linux" "arm64" "liblighter-signer.so" || echo "")
HASH_LINUX_ARM64_H=$(download_and_process_file "lighter-signer-linux-arm64.h" "linux" "arm64" "liblighter-signer.h" || echo "")

# Download Darwin arm64
HASH_DARWIN_ARM64_DYLIB=$(download_and_process_file "lighter-signer-darwin-arm64.dylib" "darwin" "arm64" "liblighter-signer.dylib" || echo "")
HASH_DARWIN_ARM64_H=$(download_and_process_file "lighter-signer-darwin-arm64.h" "darwin" "arm64" "liblighter-signer.h" || echo "")

# Download Windows amd64
HASH_WINDOWS_AMD64_DLL=$(download_and_process_file "lighter-signer-windows-amd64.dll" "windows" "amd64" "liblighter-signer.dll" || echo "")
HASH_WINDOWS_AMD64_H=$(download_and_process_file "lighter-signer-windows-amd64.h" "windows" "amd64" "liblighter-signer.h" || echo "")

# Verify files were copied
echo -e "${GREEN}Verifying files...${NC}"
MISSING_FILES=0

check_file() {
    local file="$1"
    if [[ ! -f "$file" ]]; then
        echo -e "  ${RED}Missing: $file${NC}"
        MISSING_FILES=$((MISSING_FILES + 1))
        return 1
    else
        echo -e "  ${GREEN}Found: $file${NC}"
        return 0
    fi
}

check_file "$LIBS_DIR/linux/amd64/liblighter-signer.so"
check_file "$LIBS_DIR/linux/amd64/liblighter-signer.h"
check_file "$LIBS_DIR/linux/arm64/liblighter-signer.so"
check_file "$LIBS_DIR/linux/arm64/liblighter-signer.h"
check_file "$LIBS_DIR/darwin/arm64/liblighter-signer.dylib"
check_file "$LIBS_DIR/darwin/arm64/liblighter-signer.h"
check_file "$LIBS_DIR/windows/amd64/liblighter-signer.dll"
check_file "$LIBS_DIR/windows/amd64/liblighter-signer.h"

if [[ $MISSING_FILES -gt 0 ]]; then
    echo -e "${YELLOW}Warning: $MISSING_FILES file(s) are missing. You may need to manually download them.${NC}"
fi

if [[ $HASH_VERIFICATION_FAILED -eq 1 ]]; then
    echo -e "${RED}Error: Hash verification failed for one or more files!${NC}" >&2
    echo -e "${RED}The downloaded files may be corrupted or tampered with.${NC}" >&2
    exit 1
fi

# Update README.md
echo -e "${GREEN}Updating $README_FILE...${NC}"

# Generate README content
{
    cat <<EOF
## lighter-go signing libraries

Latest version: **${TAG_NAME}**

Source: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/tag/${TAG_NAME}

### Structure

- \`linux/amd64/\` - Linux x86_64 binaries
- \`linux/arm64/\` - Linux ARM64 binaries
- \`darwin/arm64/\` - macOS ARM64 binaries
- \`windows/amd64/\` - Windows x86_64 binaries

Each directory contains:
- \`liblighter-signer.{so|dylib|dll}\` - Platform-specific library
- \`liblighter-signer.h\` - C header file

EOF

    # Add SHA256 checksums section
    echo "### SHA256 Checksums"
    echo ""
    echo "\`\`\`"
    
    # Linux amd64
    if [[ -n "$HASH_LINUX_AMD64_SO" ]]; then
        echo "${HASH_LINUX_AMD64_SO}  libs/linux/amd64/liblighter-signer.so"
    fi
    if [[ -n "$HASH_LINUX_AMD64_H" ]]; then
        echo "${HASH_LINUX_AMD64_H}  libs/linux/amd64/liblighter-signer.h"
    fi
    
    # Linux arm64
    if [[ -n "$HASH_LINUX_ARM64_SO" ]]; then
        echo "${HASH_LINUX_ARM64_SO}  libs/linux/arm64/liblighter-signer.so"
    fi
    if [[ -n "$HASH_LINUX_ARM64_H" ]]; then
        echo "${HASH_LINUX_ARM64_H}  libs/linux/arm64/liblighter-signer.h"
    fi
    
    # Darwin arm64
    if [[ -n "$HASH_DARWIN_ARM64_DYLIB" ]]; then
        echo "${HASH_DARWIN_ARM64_DYLIB}  libs/darwin/arm64/liblighter-signer.dylib"
    fi
    if [[ -n "$HASH_DARWIN_ARM64_H" ]]; then
        echo "${HASH_DARWIN_ARM64_H}  libs/darwin/arm64/liblighter-signer.h"
    fi
    
    # Windows amd64
    if [[ -n "$HASH_WINDOWS_AMD64_DLL" ]]; then
        echo "${HASH_WINDOWS_AMD64_DLL}  libs/windows/amd64/liblighter-signer.dll"
    fi
    if [[ -n "$HASH_WINDOWS_AMD64_H" ]]; then
        echo "${HASH_WINDOWS_AMD64_H}  libs/windows/amd64/liblighter-signer.h"
    fi
    
    echo "\`\`\`"
    echo ""

    cat <<EOF
### Updating

Run \`./update_libs.sh\` to download the latest binaries from GitHub releases.

The script automatically:
- Downloads the latest release assets
- Verifies SHA256 checksums using digests from GitHub API
- Updates this README with the latest version and checksums
EOF
} > "$README_FILE"

echo -e "${GREEN}✓ Update complete!${NC}"
echo -e "${GREEN}Latest version: ${TAG_NAME}${NC}"
