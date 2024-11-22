#!/usr/bin/env fish

# If cfssl or cfssljson files do not exist, download them.
if not test -f cfssl; or not test -f cfssljson
    echo (set_color blue)"📦 Downloading binaries..."(set_color normal)
else
    echo (set_color green)"✅ cfssl and cfssljson binaries already exist."(set_color normal)
    return 0
end

# Get latest tag from GitHub API.
set -l org cloudflare
set -l repo cfssl
set -l url "https://api.github.com/repos/$org/$repo/releases/latest"
# Eg: v26.0.1
set -l tag_name (curl $url -s | jq .tag_name -r)
# Eg: 26.0.1
set -l tag_name (echo $tag_name | sed "s/v//")

# Set variables.
set release_version $tag_name
set root_url https://github.com/$org/$repo/releases/download
set cfssl_bin $root_url"/v"$release_version"/cfssl_"$release_version"_linux_amd64"
set cfssljson_bin $root_url"/v"$release_version"/cfssljson_"$release_version"_linux_amd64"

# Print variables.
echo 💾 (set_color blue)"cfssl: $cfssl_bin"(set_color normal)
echo 💾 (set_color blue)"cfssljson: $cfssljson_bin"(set_color normal)

# Download binaries.
wget -q $cfssl_bin -O cfssl
wget -q $cfssljson_bin -O cfssljson

# # Make them executable.
chmod +x cfssl cfssljson
