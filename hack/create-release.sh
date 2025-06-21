#!/bin/sh

set -e

#
# Create a release from current commit
# Create GitHub release and tag with release-please
# Release build is delegated to CI
#

wait_for_release_jobs() {
  local release_tag=$1
  
  echo "Waiting for release CI jobs to finish on tag $release_tag..."
  
  local timeout=1200  # Set timeout to 20 minutes (1200 seconds)
  local start_time=$(date +%s)
  local release_jobs_success=false

  while true; do
    local current_time=$(date +%s)
    local elapsed_time=$((current_time - start_time))

    if [ $elapsed_time -ge $timeout ]; then
      echo "Timeout reached: CI jobs did not complete within $timeout seconds."
      exit 1
    fi

    # Check for jobs on release tag
    # Output is like: [ { { "name": "Release", "status": "in_progress" }]
    local release_jobs_response=$(gh run list -b "$release_tag" --json status,name)

    echo "[$(date +%Y-%m-%d-%H:%M:%S)] Release jobs response: $release_jobs_response"

    # filter for jobs with status "in_progress"
    local release_job_status=$(echo "$release_jobs_response" | jq -r '.[] | select(.name == "Release") | .status')

    echo "Release job status: '$release_job_status'"

    # If no jobs are running (release_jobs_in_progress is an empty string), break: all release jobs completed
    if [ "$release_job_status" = "completed" ]; then
      echo "Release CI job completed for $release_tag"
      release_jobs_success=true
      break
    else
      echo "CI jobs still running for $release_tag. Waiting..."
      sleep 30
    fi
  done
}

if [ -z ${GITHUB_TOKEN+x} ]; then 
    echo "GITHUB_TOKEN variable must be set (with read/write permissions on content and pull requests)"
    exit 1
fi

echo "Current commit message:"
echo "---"
git log -1 --pretty=%B | cat
echo "---"
echo

echo "Create release last PR?"
read -p "'yes' to continue: " answer

case ${answer:-N} in
    yes ) echo "🚀";;
    * ) echo "Type 'yes' to continue"; exit 1;;
esac

# Create release draft
# Despite passing prerelease=true, release-please github-release will publish as latest...
# Ensure release is prerelease with subsequent command
npx release-please github-release --repo-url https://github.com/PierreBeucher/novops --token=${GITHUB_TOKEN} --prerelease true

current_release=$(gh release list -L 1 | cut -d$'\t' -f1)

echo "Found release: $current_release - marking it as prerelease"

# Ensure release is prerelease
gh release edit "${current_release}" --prerelease

# Wait for release CI jobs to finish (upload all artifacts)
wait_for_release_jobs "$current_release"

# Finalize it !
gh release edit "${current_release}" --latest --prerelease=false