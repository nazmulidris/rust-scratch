#!/usr/bin/env fish

function main
    # Eg: https://api.github.com/repos/rubjo/victor-mono/releases/latest
    set -l org UltimateHackingKeyboard
    set -l repo agent
    set -l url "https://api.github.com/repos/$org/$repo/releases/latest"
    # Eg: v26.0.1
    set -l tag_name (curl $url -s | jq .tag_name -r)
    # Eg: 26.0.1
    set -l tag_name (echo $tag_name | sed "s/v//")

    set -l download_file "UHK.Agent-$tag_name-linux-x86_64.AppImage"
    # Eg: https://github.com/UltimateHackingKeyboard/agent/releases/download/v4.2.0/UHK.Agent-4.2.0-linux-x86_64.AppImage
    set -l download_url "https://github.com/$org/$repo/releases/download/v$tag_name/$download_file"

    echo (set_color blue)"download_file: " $download_file(set_color normal)
    echo (set_color blue)"download_url:  " $download_url(set_color normal)

    set -l download_folder "$HOME/Downloads/uhk-agent-tmp"
    if test -d $download_folder
        echo "Removing existing $download_folder directory"
        rm -rf $download_folder
    end
    echo "Creating $download_folder directory"
    mkdir -p $download_folder

    pushd $download_folder
    echo "Downloading $download_file to $download_folder"
    sh -c "wget -q -O $download_file $download_url"

    echo "Extracting $download_file in $download_folder"
    chmod +x $download_file
    ./$download_file --appimage-extract >/dev/null

    echo "Renaming squashfs-root to uhk-agent"
    mv squashfs-root uhk-agent

    # If ~/bin does not exist, create it.
    if not test -d ~/bin
        echo "Creating ~/bin"
        mkdir ~/bin
    end

    # Remove any existing uhk-agent directory.
    if test -d ~/bin/uhk-agent/
        echo "Removing existing uhk-agent directory in ~/bin"
        rm -rf ~/bin/uhk-agent/
    end

    # Copy the extracted uhk-agent directory to ~/bin.
    echo "Copying uhk-agent to ~/bin"
    cp -r uhk-agent ~/bin
    echo "Copying the UHK Agent icon to ~/bin"
    cp ~/bin/uhk-agent/uhk-agent.png ~/bin

    # Create the desktop file.
    echo "Writing the uhk_agent.desktop file"
    _write_desktop_file

    popd
end

set output_file "$HOME/.local/share/applications/uhk-agent.desktop"

function _write_desktop_file
    echo >$output_file "
[Desktop Entry]
Type=Application
Name=UHK Agent
Comment=Launch UHK Agent
Categories=Utilities;
Icon=/home/$USER/bin/uhk-agent.png
Exec=/home/$USER/bin/uhk-agent/uhk-agent --no-sandbox
Terminal=false
"
end

# Run the script.
main
