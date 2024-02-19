#!/bin/sh

set -e

#
# Create a release from current commit
# Build all cross-platforms binaries locally 
# and create releases with artifacts
# 
# Must be run after release-please PR has been merged 
# If script fails, it can be restarted safely and should be reasonably idempotent
#

if [ -z ${var+x} ]; then 
    echo "GITHUB_TOKEN variable must be set (with read/write permissions on content and pull requests)"
else 
    echo "var is set to '$var'"
fi

echo "Current commit message:"
echo "---"
git log -1 --pretty=%B | cat
echo "---"
echo

echo "Create release for from current commit?"
read -p "'yes' to continue: " answer

case ${answer:-N} in
    yes ) echo "🚀";;
    * ) echo "Type 'yes' to continue"; exit 1;;
esac

function build_and_zip() {
    local target_name="$1"
    local artifact_suffix="$2"

    # Use different target dir to avoid glibc version error
    # See https://github.com/cross-rs/cross/issues/724
    local target_dir="target/cross/${target_name}"
    local novops_binary="${target_dir}/${target_name}/release/novops"
    cross build --target "${target_name}" --target-dir "${target_dir}" --release

    # zip artifact + sha
    zip -j "release/novops${artifact_suffix}.zip" "${novops_binary}"
    sha256sum "${novops_binary}" > "release/novops${artifact_suffix}.sha256sum"
}

# cleanup before packaging release
rm -r release || true
mkdir release

build_and_zip x86_64-unknown-linux-musl     "_linux_x86_64"
build_and_zip aarch64-unknown-linux-musl    "_linux_aarch64"
build_and_zip x86_64-apple-darwin           "_macos_x86_64"
build_and_zip aarch64-apple-darwin          "_macos_aarch64"

# Legacy release name using "novops-X64-Linux.zip" 
# to avoid disruption on install scripts relying on this release name
build_and_zip x86_64-unknown-linux-musl     "-X64-Linux"

# Create release draft
npx release-please github-release --repo-url https://github.com/PierreBeucher/novops --token=${GITHUB_TOKEN} --draft

current_release=$(gh release list -L 1 | cut -d$'\t' -f1)
echo "Upload artifacts for release '${current_release}'"
read -p "'yes' to continue: " answer
case ${answer:-N} in
    yes ) echo "Uploading artifacts...";;
    * ) echo "Type 'yes' to continue"; exit 1;;
esac

# make sure release is draft (normally it's ok but release-please may ignore draft)
gh release edit "${current_release}" --draft

# Upload all artifacts
gh release upload "${current_release}" release/*

# Finalize it !
gh release edit "${current_release}" --latest --draft=false