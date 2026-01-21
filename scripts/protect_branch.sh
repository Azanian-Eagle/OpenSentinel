#!/bin/bash
set -e

# Configuration
BRANCH="master"
REQUIRED_CONTEXTS=("test" "security-audit" "lint")

# Check for GITHUB_TOKEN
if [ -z "$GITHUB_TOKEN" ]; then
  echo "Error: GITHUB_TOKEN environment variable is not set."
  echo "Please export a GitHub Personal Access Token with 'repo' scope."
  exit 1
fi

# Get Repo Info
REPO_URL=$(git remote get-url origin)
# Extract owner/repo from https://github.com/owner/repo or git@github.com:owner/repo.git
if [[ "$REPO_URL" =~ github\.com[:/]([^/]+)/(.+)(\.git)?$ ]]; then
    OWNER=${BASH_REMATCH[1]}
    REPO=${BASH_REMATCH[2]}
    # Remove .git suffix if captured in REPO group (regex above might be greedy)
    REPO=${REPO%.git}
else
    echo "Error: Could not determine repository owner and name from remote URL: $REPO_URL"
    exit 1
fi

echo "Configuring branch protection for $OWNER/$REPO branch: $BRANCH..."

# Construct JSON payload
# Note: users/teams/apps arrays in restrictions are empty to disable push access for everyone except admins (if enforce_admins is false) or restricted properly.
# However, usually "restrictions: null" means no restrictions on who can push (besides write access).
# To strictly follow "Protect this branch from force pushing", allow_force_pushes: false (which is default in the API if not specified? No, strictly part of the API).
# Wait, the v3 API for protection is PUT /repos/{owner}/{repo}/branches/{branch}/protection
# Parameters:
# - required_status_checks
# - enforce_admins
# - required_pull_request_reviews
# - restrictions (pass null to disable specific push restrictions, relying on write access)

# To prevent force pushes and deletions, those are implicit in having protection ON, unless overridden (but there is a separate endpoint for boolean flags in newer API, but main protection endpoint covers most).
# Actually, allow_force_pushes and allow_deletions are separate fields in the input for creating/updating protection in GraphQL, but in REST they are often part of the payload or separate.
# In REST v3, `allow_force_pushes` and `allow_deletions` default to false when protection is enabled.

CONTEXTS_JSON=$(printf '"%s",' "${REQUIRED_CONTEXTS[@]}")
CONTEXTS_JSON="[${CONTEXTS_JSON%,}]"

PAYLOAD=$(cat <<EOF
{
  "required_status_checks": {
    "strict": true,
    "contexts": $CONTEXTS_JSON
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": true,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF
)

RESPONSE=$(curl -s -X PUT \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  -d "$PAYLOAD" \
  "https://api.github.com/repos/$OWNER/$REPO/branches/$BRANCH/protection")

# Check if successful (look for url field in response or check HTTP code)
if echo "$RESPONSE" | grep -q '"url":'; then
  echo "Success! Branch protection enabled for $BRANCH."
else
  echo "Failed to enable branch protection."
  echo "Response: $RESPONSE"
  exit 1
fi

# Also require signed commits (separate endpoint)
echo "Enabling signed commits requirement..."
RESPONSE_SIG=$(curl -s -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  -d '{"enabled": true}' \
  "https://api.github.com/repos/$OWNER/$REPO/branches/$BRANCH/protection/required_signatures")

if echo "$RESPONSE_SIG" | grep -q '"enabled": true' || echo "$RESPONSE_SIG" | grep -q '"url":'; then
   echo "Signed commits requirement enabled."
else
   echo "Warning: Could not enable signed commits requirement. (Feature might not be available on all plans)"
   echo "Response: $RESPONSE_SIG"
fi
