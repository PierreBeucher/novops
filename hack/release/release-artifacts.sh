#!/bin/sh

set -e

# Check if current commit is tag
current_tags=$(git tag --contains HEAD)
current_tag_count=$(echo "$current_tags" | wc -l | tr -d ' '); 
echo "$([ ! -z "$current_tag_count" ])"
if [ ! -z "$current_tags" ] && [ "$current_tag_count" -eq 1 ]; then 
    
    echo "Create release for '$current_tags'?"
    read -p "'yes' to continue: " answer

    case ${answer:-N} in
        yes ) echo "🚀";;
        * ) echo "Type 'yes' to continue"; exit 1;;
    esac
else 
    echo "Not exactly 1 tag on current commit. Found: ${current_tags}"
    exit 2
fi

function build_and_zip() {
    local target_name="$1"
    local artifact_suffix="$2"

    # Use different target dir to avoid glibc version error
    # See https://github.com/cross-rs/cross/issues/724
    local target_dir="target/cross/${target_name}"
    local novops_binary="${target_dir}/${target_name}/release/novops"
    cross build --target "${target_name}" --target-dir "${target_dir}" --release

    # zip artifact + sha
    zip -j "release/novops-${artifact_suffix}.zip" "${novops_binary}"
    sha256sum "${novops_binary}" > "release/novops-${artifact_suffix}.sha256sum"
}

# cleanup before packaging release
rm -r release || true
mkdir release

build_and_zip x86_64-unknown-linux-musl     linux_x86_64
build_and_zip aarch64-unknown-linux-musl    linux_aarch64
build_and_zip x86_64-apple-darwin           macos_x86_64
build_and_zip aarch64-apple-darwin          macos_aarch64

# Legacy release name using "novops-X64-Linux.zip" 
# to avoid disruption on install scripts relying on this release name
build_and_zip x86_64-unknown-linux-musl X64-Linux

# gh release upload ${GITHUB_REF_NAME} \
#   build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip \
#   build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip.sha256sum

