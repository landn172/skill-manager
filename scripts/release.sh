#!/bin/bash

# Release script - Updates version, creates git tag, and pushes
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.2.0

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "❌ Please provide a version number"
  echo "Usage: npm run release <version>"
  echo "Example: npm run release 0.2.0"
  exit 1
fi

# Validate version format (semver-like)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "❌ Invalid version format. Use semantic versioning (e.g., 0.2.0)"
  exit 1
fi

echo "🚀 Releasing version $VERSION..."

# Update package.json version
echo "📦 Updating package.json..."
npm version "$VERSION" --no-git-tag-version

# Update tauri.conf.json version
echo "🦀 Updating tauri.conf.json..."
TAURI_CONF="src-tauri/tauri.conf.json"
if [[ "$OSTYPE" == "darwin"* ]]; then
  # macOS
  sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$TAURI_CONF"
else
  # Linux
  sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$TAURI_CONF"
fi

# Update Cargo.toml version
echo "📦 Updating Cargo.toml..."
CARGO_TOML="src-tauri/Cargo.toml"
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" "$CARGO_TOML"
else
  sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" "$CARGO_TOML"
fi

# Update Cargo.lock version for skill-manager package
echo "🔒 Updating Cargo.lock..."
CARGO_LOCK="src-tauri/Cargo.lock"
if [[ "$OSTYPE" == "darwin"* ]]; then
  # Use awk to update version only for skill-manager package
  awk -v version="$VERSION" '
    /^\[\[package\]\]/ { in_skill_manager = 0 }
    /^name = "skill-manager"/ { in_skill_manager = 1 }
    in_skill_manager && /^version = / { 
      print "version = \"" version "\""
      next 
    }
    { print }
  ' "$CARGO_LOCK" > "${CARGO_LOCK}.tmp" && mv "${CARGO_LOCK}.tmp" "$CARGO_LOCK"
else
  awk -v version="$VERSION" '
    /^\[\[package\]\]/ { in_skill_manager = 0 }
    /^name = "skill-manager"/ { in_skill_manager = 1 }
    in_skill_manager && /^version = / { 
      print "version = \"" version "\""
      next 
    }
    { print }
  ' "$CARGO_LOCK" > "${CARGO_LOCK}.tmp" && mv "${CARGO_LOCK}.tmp" "$CARGO_LOCK"
fi

# Git operations
echo "📝 Committing changes..."
git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: release v$VERSION"

echo "🏷️  Creating tag v$VERSION..."
git tag "v$VERSION"

echo "📤 Pushing to origin..."
git push origin main
git push origin "v$VERSION"

echo ""
echo "✅ Successfully released v$VERSION!"
echo "🔗 GitHub Actions will now build and create the release."
echo "   Check: https://github.com/landn172/skill-manager/actions"
