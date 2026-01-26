#!/bin/bash
# ============================================================================
# Setup Git Hooks
# ============================================================================
# Run this script once to install Git hooks for the project
# Usage: ./setup-hooks.sh
# ============================================================================

echo "🔧 Setting up Git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy pre-commit hook
if [ -f "hooks/pre-commit" ]; then
    cp hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✅ Pre-commit hook installed"
else
    echo "❌ hooks/pre-commit not found"
    exit 1
fi

# Copy pre-push hook if it exists
if [ -f "hooks/pre-push" ]; then
    cp hooks/pre-push .git/hooks/pre-push
    chmod +x .git/hooks/pre-push
    echo "✅ Pre-push hook installed"
fi

echo ""
echo "🎉 Git hooks setup complete!"
echo ""
echo "Hooks will now run automatically:"
echo "  - pre-commit: Runs before each commit (lint, format, test)"
echo ""
echo "To skip hooks temporarily (not recommended):"
echo "  git commit --no-verify"
echo ""
echo "To run checks manually:"
echo "  cd monitor/backend"
echo "  cargo fmt         # Format code"
echo "  cargo clippy      # Run linter"
echo "  cargo test        # Run tests"
