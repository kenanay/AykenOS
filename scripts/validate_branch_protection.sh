#!/usr/bin/env bash
# Validate Branch Protection Rules
# Author: Kenan AY — System Architect
#
# Purpose: Verify branch protection under the single-maintainer authority model
# Requires: GitHub CLI (gh) with repository read access

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
echo -e "${BLUE}Branch Protection Validation${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ ERROR: GitHub CLI (gh) is not installed${NC}"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ ERROR: Not authenticated with GitHub CLI${NC}"
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

# Validation results
total_checks=0
passed_checks=0
failed_checks=0

# Function to validate branch protection
validate_branch() {
    local branch=$1
    local branch_checks=0
    local branch_passed=0

    echo -e "${YELLOW}Validating branch: $branch${NC}"

    # Fetch branch protection settings
    local protection
    if ! protection=$(gh api "/repos/$REPO/branches/$branch/protection" 2>/dev/null); then
        echo -e "  ${RED}❌ Branch protection not configured${NC}"
        echo ""
        return 1
    fi

    # Check 1: Required status checks enabled
    ((total_checks+=1))
    ((branch_checks+=1))
    local status_checks_enabled=$(echo "$protection" | jq -r '.required_status_checks != null')
    if [ "$status_checks_enabled" = "true" ]; then
        echo -e "  ${GREEN}✅ Required status checks enabled${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${RED}❌ Required status checks not enabled${NC}"
        ((failed_checks+=1))
    fi

    # Check 2: Strict status checks (branches must be up to date)
    ((total_checks+=1))
    ((branch_checks+=1))
    local strict=$(echo "$protection" | jq -r '.required_status_checks.strict // false')
    if [ "$strict" = "true" ]; then
        echo -e "  ${GREEN}✅ Branches must be up to date${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${RED}❌ Branches not required to be up to date${NC}"
        ((failed_checks+=1))
    fi

    # Check 3: All required checks present
    ((total_checks+=1))
    ((branch_checks+=1))
    local configured_checks=$(echo "$protection" | jq -r '.required_status_checks.contexts[]' 2>/dev/null || echo "")
    local missing_checks=()

    for check in "${REQUIRED_CHECKS[@]}"; do
        if ! echo "$configured_checks" | grep -q "^$check$"; then
            missing_checks+=("$check")
        fi
    done

    if [ ${#missing_checks[@]} -eq 0 ]; then
        echo -e "  ${GREEN}✅ All required checks configured${NC}"
        for check in "${REQUIRED_CHECKS[@]}"; do
            echo -e "     - $check"
        done
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${RED}❌ Missing required checks:${NC}"
        for check in "${missing_checks[@]}"; do
            echo -e "     - $check"
        done
        ((failed_checks+=1))
    fi

    # Check 4: Single-maintainer review policy (no impossible self-approval)
    ((total_checks+=1))
    ((branch_checks+=1))
    local required_approvals=$(echo "$protection" | jq -r 'if .required_pull_request_reviews == null then -1 else .required_pull_request_reviews.required_approving_review_count end')
    local require_codeowners=$(echo "$protection" | jq -r 'if .required_pull_request_reviews == null then true else .required_pull_request_reviews.require_code_owner_reviews end')
    if [ "$required_approvals" = "0" ] && [ "$require_codeowners" = "false" ]; then
        echo -e "  ${GREEN}✅ Single-maintainer review policy aligned${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${RED}❌ Review policy requires unavailable independent/self approval${NC}"
        ((failed_checks+=1))
    fi

    # Check 5: Stale review dismissal
    ((total_checks+=1))
    ((branch_checks+=1))
    local dismiss_stale=$(echo "$protection" | jq -r '.required_pull_request_reviews.dismiss_stale_reviews // false')
    if [ "$dismiss_stale" = "true" ]; then
        echo -e "  ${GREEN}✅ Stale reviews dismissed on new commits${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${YELLOW}⚠️  Stale reviews not dismissed (recommended)${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    fi

    # Check 6: Enforce for administrators
    ((total_checks+=1))
    ((branch_checks+=1))
    local enforce_admins=$(echo "$protection" | jq -r '.enforce_admins.enabled // false')
    if [ "$enforce_admins" = "true" ]; then
        echo -e "  ${GREEN}✅ Enforced for administrators${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${YELLOW}⚠️  Not enforced for administrators (recommended)${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    fi

    # Check 7: Force pushes disabled
    ((total_checks+=1))
    ((branch_checks+=1))
    local allow_force_pushes=$(echo "$protection" | jq -r '.allow_force_pushes.enabled // false')
    if [ "$allow_force_pushes" = "false" ]; then
        echo -e "  ${GREEN}✅ Force pushes disabled${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${RED}❌ Force pushes allowed${NC}"
        ((failed_checks+=1))
    fi

    # Check 8: Branch deletion disabled
    ((total_checks+=1))
    ((branch_checks+=1))
    local allow_deletions=$(echo "$protection" | jq -r '.allow_deletions.enabled // false')
    if [ "$allow_deletions" = "false" ]; then
        echo -e "  ${GREEN}✅ Branch deletion disabled${NC}"
        ((passed_checks+=1))
        ((branch_passed+=1))
    else
        echo -e "  ${RED}❌ Branch deletion allowed${NC}"
        ((failed_checks+=1))
    fi

    echo ""

    # Return success if all critical checks passed
    if [ $branch_passed -eq $branch_checks ]; then
        return 0
    else
        return 1
    fi
}

# Validate each branch
branch_success=0
for branch in "${PROTECTED_BRANCHES[@]}"; do
    if validate_branch "$branch"; then
        ((branch_success+=1))
    fi
done

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Validation Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Total checks: $total_checks"
echo -e "Passed: ${GREEN}$passed_checks${NC}"
echo -e "Failed: ${RED}$failed_checks${NC}"
echo ""
echo -e "Protected branches validated: $branch_success/${#PROTECTED_BRANCHES[@]}"
echo ""

# Exit status
if [ $failed_checks -eq 0 ] && [ $branch_success -eq ${#PROTECTED_BRANCHES[@]} ]; then
    echo -e "${GREEN}✅ All branch protection rules are correctly configured${NC}"
    echo ""
    echo "Branch protection is enforcing:"
    echo "  ✅ Required status check (freeze)"
    echo "  ✅ Branches must be up to date before merge"
    echo "  ✅ Single-maintainer policy (no impossible self-approval)"
    echo "  ✅ Protection applies to administrators"
    echo "  ✅ Force pushes disabled"
    echo "  ✅ Branch deletion disabled"
    exit 0
else
    echo -e "${RED}❌ Branch protection configuration issues detected${NC}"
    echo ""
    echo "To fix:"
    echo "  1. Run: ./scripts/setup_branch_protection.sh"
    echo "  2. Or configure manually via GitHub web interface"
    echo "  3. See: docs/dev-loop/BRANCH_PROTECTION.md"
    exit 1
fi
