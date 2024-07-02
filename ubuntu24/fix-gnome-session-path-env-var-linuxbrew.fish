#!/usr/bin/env fish

set searchString linuxbrew
set dotProfileFile "$HOME/.profile"

# Deal with
# 1. linuxbrew added to path
# 2. cargo added to path
# 3. API keys for various services
function _writeToDotProfileFile
    echo >>$dotProfileFile '
# Add linuxbrew to path.
if [ -d "/home/linuxbrew/.linuxbrew" ] ; then
  PATH="/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:$PATH"
fi

# Load Rust cargo env vars.
if [ -f "$HOME/.cargo/env" ] ; then
  . "$HOME/.cargo/env"
fi
'
end

function addLinuxbrewToGnomeSessionPath
    set -l grepResponse (grep $searchString $dotProfileFile)

    # echo "grepResponse: '$grepResponse'"

    if test -n "$grepResponse"
        echo "$searchString found in $dotProfileFile, nothing to do."
    else
        echo "$searchString not found in $dotProfileFile, make sure to add it."
        _writeToDotProfileFile
    end
end

addLinuxbrewToGnomeSessionPath
