#!/usr/bin/env bash
# Deploy the "github-pages" artifact to GitHub Pages.
#
# This replicates the API sequence of actions/deploy-pages:
#   1. resolve the artifact ID for this run
#   2. obtain an OIDC token (core.getIDToken(), no audience)
#   3. POST /repos/{nwo}/pages/deployments
#   4. poll the deployment status until it settles
#
# Why not actions/deploy-pages directly? It resolves the artifact ID through
# the Actions Results (Twirp) service, which intermittently returns an empty
# artifact list even right after a successful upload (GitHub-side incident,
# 2026-11-25: "No artifacts named github-pages were found" while the REST API
# listed the artifact). The REST API is a separate backend and lists the
# artifact reliably, so we resolve the ID there and drive the same deployment
# endpoints the official action uses.
#
# Required environment:
#   GITHUB_REPOSITORY, GITHUB_RUN_ID, GITHUB_SHA (provided by the runner)
#   GH_TOKEN with pages:write
#   ACTIONS_ID_TOKEN_REQUEST_TOKEN / ACTIONS_ID_TOKEN_REQUEST_URL (requires
#   the job to have the id-token: write permission)
set -euo pipefail

nwo="${GITHUB_REPOSITORY}"
sha="${GITHUB_SHA}"
artifact_name="${MARMITE_ARTIFACT_NAME:-github-pages}"
max_wait_seconds="${MARMITE_ARTIFACT_WAIT:-120}"

# 1. Resolve the artifact ID from the Actions REST API, retrying to absorb
#    eventual consistency of the listing endpoint.
artifact_id=""
deadline=$(( $(date +%s) + max_wait_seconds ))
while true; do
  artifact_id=$(gh api "repos/${nwo}/actions/runs/${GITHUB_RUN_ID}/artifacts?per_page=100" \
    --jq ".artifacts[] | select(.name == \"${artifact_name}\") | .id" || true)
  if [ -n "$artifact_id" ]; then
    break
  fi
  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "::error::Artifact '${artifact_name}' not visible to the REST API after ${max_wait_seconds}s"
    exit 1
  fi
  echo "Artifact '${artifact_name}' not visible yet; retrying in 5s"
  sleep 5
done
echo "Deploying artifact ${artifact_id} (build ${sha})"

# 2. OIDC token, identical to core.getIDToken() with no audience.
id_token=$(curl -sS -H "Authorization: bearer ${ACTIONS_ID_TOKEN_REQUEST_TOKEN}" \
  "${ACTIONS_ID_TOKEN_REQUEST_URL}" | jq -r '.value')
if [ -z "$id_token" ] || [ "$id_token" = "null" ]; then
  echo "::error::Failed to obtain OIDC token (job needs id-token: write)"
  exit 1
fi

# 3. Create the Pages deployment (same payload as deploy-pages).
request_body=$(mktemp)
trap 'rm -f "$request_body"' EXIT
jq -n --argjson artifact_id "$artifact_id" --arg sha "$sha" --arg token "$id_token" \
  '{artifact_id: $artifact_id, pages_build_version: $sha, oidc_token: $token}' > "$request_body"
deployment_id=$(gh api --method POST "repos/${nwo}/pages/deployments" \
  --input "$request_body" --jq '.id')
echo "id=${deployment_id}" >> "${GITHUB_OUTPUT:-/dev/null}"
echo "Created deployment ${deployment_id}"

# 4. Poll until the deployment settles (deploy-pages allows 10 minutes).
deadline=$(( $(date +%s) + 600 ))
while true; do
  sleep 5
  status=$(gh api "repos/${nwo}/pages/deployments/${deployment_id}" --jq '.status' 2>/dev/null \
    || echo "unknown_status")
  echo "Status: ${status}"
  case "$status" in
    succeed)
      echo "status=succeed" >> "${GITHUB_OUTPUT:-/dev/null}"
      echo "Deployment succeeded"
      exit 0
      ;;
    deployment_failed | deployment_content_failed | deployment_cancelled | deployment_lost)
      echo "::error::Deployment ended with status: ${status}"
      exit 1
      ;;
  esac
  # updating_pages, deployment_attempt_error, not_found, unknown_status are
  # treated as transient, matching deploy-pages.
  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "Timed out waiting for the deployment; cancelling"
    gh api --method POST "repos/${nwo}/pages/deployments/${deployment_id}/cancel" >/dev/null 2>&1 || true
    echo "::error::Deployment timed out after 600s"
    exit 1
  fi
done
