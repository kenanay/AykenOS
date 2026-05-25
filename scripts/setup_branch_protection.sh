#!/usr/bin/env bash
# Setup Branch Protection Rules
# Author: Kenan AY — System Architect
#
# Purpose: Configure GitHub branch protection rules for constitutional CI
# Requires: GitHub CLI (gh) with repository admin access

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROTECTED_BRANCHES=("main")
REQUIRED_CHECKS=("freeze")

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Branch Protection Setup${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ ERROR: GitHub CLI (gh) is not installed${NC}"
    echo ""
    echo "Install with:"
    echo "  macOS:   brew install gh"
    echo "  Linux:   See https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
    echo "  Windows: See https://github.com/cli/cli/releases"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ ERROR: Not authenticated with GitHub CLI${NC}"
    echo ""
    echo "Authenticate with:"
    echo "  gh auth login"
    exit 1
fi

# Get repository info
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "")
if [ -z "$REPO" ]; then
    echo -e "${RED}❌ ERROR: Not in a GitHub repository${NC}"
    exit 1
fi

echo -e "${GREEN}Repository: $REPO${NC}"
echo ""

# Function to setup branch protection for a branch
setup_protection() {
    local branch=$1

    echo -e "${YELLOW}Configuring protection for branch: $branch${NC}"

    # Build required checks argument
    local checks_arg=""
    for check in "${REQUIRED_CHECKS[@]}"; do
        checks_arg="$checks_arg --required-status-check \"$check\""
    done

    # Note: GitHub CLI doesn't support all branch protection settings
    # We'll use the API directly for full control

    # Create JSON payload
    local payload=$(cat <<EOF
{
  "required_status_checks": {
    "strict": true,
    "contexts": $(printf '%s\n' "${REQUIRED_CHECKS[@]}" | jq -R . | jq -s .)
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "require_last_push_approval": false,
    "required_approving_review_count": 0
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_linear_history": false,
  "required_conversation_resolution": false
}
EOF
)

    # Apply branch protection using GitHub API
    if gh api \
        --method PUT \
        -H "Accept: application/vnd.github+json" \
        "/repos/$REPO/branches/$branch/protection" \
        --input - <<< "$payload" &> /dev/null; then
        echo -e "  ${GREEN}✅ Protection configured${NC}"
    else
        echo -e "  ${RED}❌ Failed to configure protection${NC}"
        echo -e "  ${YELLOW}Note: You may need admin permissions${NC}"
        return 1
    fi

    echo ""
}

# Setup protection for each branch
success_count=0
for branch in "${PROTECTED_BRANCHES[@]}"; do
    if setup_protection "$branch"; then
        ((success_count+=1))
    fi
done

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Protected branches: ${#PROTECTED_BRANCHES[@]}"
echo -e "Successfully configured: $success_count"
echo ""

if [ $success_count -eq ${#PROTECTED_BRANCHES[@]} ]; then
    echo -e "${GREEN}✅ All branch protection rules configured successfully${NC}"
    echo ""
    echo "Required status checks:"
    for check in "${REQUIRED_CHECKS[@]}"; do
        echo "  - $check"
    done
    echo ""
    echo "Settings applied:"
    echo "  ✅ Require status checks to pass"
    echo "  ✅ Require branches to be up to date"
    echo "  ✅ Single-maintainer review policy (no impossible self-approval)"
    echo "  ✅ CODEOWNERS remains accountability metadata"
    echo "  ✅ Enforce for administrators"
    echo "  ✅ Prevent force pushes"
    echo "  ✅ Prevent branch deletion"
    echo ""
    echo "Verify with:"
    echo "  ./scripts/validate_branch_protection.sh"
    exit 0
else
    echo -e "${RED}❌ Some branch protection rules failed to configure${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Verify you have admin permissions on the repository"
    echo "  2. Check that branches exist (create them if needed)"
    echo "  3. Verify GitHub CLI authentication: gh auth status"
    echo "  4. Try configuring manually via GitHub web interface"
    exit 1
fi
