#!/bin/bash
# Dockerfile hardening verification
# This script checks that the Dockerfile follows security best practices.
# It should pass after the Dockerfile is hardened.

set -e

DOCKERFILE="Dockerfile"

echo "=== Dockerfile Hardening Checks ==="

# Check 1: Non-root user is set
if grep -q "^USER" "$DOCKERFILE"; then
    echo "✓ USER directive found"
else
    echo "✗ FAIL: No USER directive — container should run as non-root"
    exit 1
fi

# Check 2: Base images use pinned versions (not :latest)
if grep -qE "FROM\s+\S+:latest" "$DOCKERFILE"; then
    echo "✗ FAIL: Found :latest tag — base images must be pinned"
    exit 1
else
    echo "✓ No :latest tags found"
fi

# Check 3: Multi-stage build exists
stage_count=$(grep -c "^FROM" "$DOCKERFILE")
if [ "$stage_count" -ge 2 ]; then
    echo "✓ Multi-stage build with $stage_count stages"
else
    echo "✗ FAIL: Not a multi-stage build"
    exit 1
fi

# Check 4: .dockerignore exists and excludes target
if [ -f ".dockerignore" ] && grep -q "target" ".dockerignore"; then
    echo "✓ .dockerignore excludes target/"
else
    echo "✗ FAIL: .dockerignore missing or doesn't exclude target/"
    exit 1
fi

# Check 5: No sudo or apt-get (alpine uses apk)
if grep -qE "(sudo|apt-get)" "$DOCKERFILE"; then
    echo "✗ FAIL: Found sudo or apt-get — use alpine packages"
    exit 1
else
    echo "✓ No sudo/apt-get found"
fi

# Check 6: Runner stage creates non-root user before switching
if grep -qE "(adduser|adduser|createuser)" "$DOCKERFILE" && grep -q "^USER" "$DOCKERFILE"; then
    echo "✓ Non-root user created"
else
    echo "✗ FAIL: No non-root user creation"
    exit 1
fi

echo ""
echo "=== All Dockerfile hardening checks passed ==="
