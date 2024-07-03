#!/usr/bin/env fish

# Get sudo permissions immediately. Fail if can't get them. This must be done before
# `utils.fish` can be loaded.
# - https://fishshell.com/docs/current/faq.html#how-do-i-get-the-exit-status-of-a-command
sudo echo "👋 Welcome to fresh-install.fish"
if not test $status -eq 0
    echo "⛔ Can't proceed without sudo permissions, exiting"
    exit
end

# Make sure that sudo permissions have been acquired before loading this script.

# Constants.

set aptInstallCmd "sudo apt install -y"
set aptUpdateCmd "sudo apt update -y"
set aptUpgradeCmd "sudo apt upgrade -y"
set aptAutoremoveCmd "sudo apt autoremove -y"
set aptAddRepositoryCmd "sudo apt-add-repository -y"
set aptAddKeyCmd "sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys"
set snapInstallCmd "sudo snap install"

# Utility functions.
function prompt_or_exit
    read -P "continue [Y/n]: " response
    if test $response = n
        echo "🖖 Exiting!"
        exit
    end
end

function executeString
    sh -c "$argv"
end

# Params: snap packages to install.
function snapInstall
    executeString "$snapInstallCmd $argv"
end

# Params: apt packages to install.
function aptInstall
    executeString "$aptInstallCmd $argv"
end

# Params: PPA repository to add to apt.
function aptAddRepository
    executeString "$aptAddRepositoryCmd $argv"
end

# Params: A list of keys that should be added.
function addAptKeys
    set count 0
    for key in $argv
        set count (math $count+1)
        # echo "[$count] key: $key"
        executeString "$aptAddKeyCmd $key"
    end
end

function aptUpdate
    executeString "$aptUpdateCmd"
end

function aptUpgrade
    executeString "$aptUpgradeCmd"
end

function aptAutoremove
    executeString "$aptAutoremoveCmd"
end

# Save the $url string to the /etc/apt/sources.list.d/$filename. Both params may have spaces.
# It actually generates something like the following:
# sudo sh -c 'echo "deb http://apt.insync.io/ubuntu bionic non-free contrib" > /etc/apt/sources.list.d/insync.list'
# sudo sh -c 'echo "deb http://dell.archive.canonical.com/updates/ bionic-dell-beaver-osp1-ellaria public" > /etc/apt/sources.list.d/bionic-dell-beaver-osp1-ellaria.list'
# More info:
# - https://itsfoss.com/ppa-guide/
# - https://linuxize.com/post/how-to-list-installed-packages-on-ubuntu/
# - https://help.ubuntu.com/community/MetaPackages
function writeAptSourceListFileAsRoot -a stringContent filename distroCodename
    set filepath /etc/apt/sources.list.d
    # set command "echo $stringContent | sudo tee $filepath/$filename"
    set command "sudo sh -c 'echo \"$stringContent\" > $filepath/$filename'"
    executeString $command
end

# Return a string that has the distro name, eg: "bionic" or "focal".
# - Delete space using tr or sed: https://askubuntu.com/a/538771/872482
# - How to use lsb_release: https://www.cyberciti.biz/faq/find-linux-distribution-name-version-number/
function getDistroCodename
    # lsb_release -c | sed -e 's/Codename:[[:space:]]//g'
    set codename (lsb_release -c | tr -d '[:blank:]' | sed -e 's/Codename://g')
    echo $codename
end

# Return the model name and number of the hardware, eg: "Precision3440".
# - https://unix.stackexchange.com/questions/254599/how-to-get-the-computer-name-not-its-hostname
function getMachineName
    sudo dmidecode | grep -A3 '^System Information' | grep Product | tr -d '[:blank:]' | sed -e 's/ProductName://g'
end

function printHeader
    echo (set_color brmagenta) "┬──────────────────────────────────────────────────────────────────────────────────────────────────"
    echo (set_color brmagenta) "│ 🐵 $argv ..."
    echo (set_color brmagenta) "┴──────────────────────────────────────────────────────────────────────────────────────────────────" (set_color normal)
end

# More info on prompting a user for confirmation using fish read function: https://stackoverflow.com/a/16673745/2085356
# More info about fish `read` function: https://fishshell.com/docs/current/cmds/read.html
function _promptUserForConfirmation -a message
    if not test -z "$message"
        echo (set_color brmagenta)"🤔 $message?"
    end

    while true
        # read -l -P '🔴 Do you want to continue? [y/N] ' confirm
        read -l -p "set_color brcyan; echo '🔴 Do you want to continue? [y/N] ' ; set_color normal; echo '> '" confirm
        switch $confirm
            case Y y
                return 0
            case '' N n
                return 1
        end
    end
end

# Constants derived from utility functions.

set distroCodename (getDistroCodename)
set machineName (getMachineName)

function main
    printHeader "\$distroCodename: '$distroCodename', \$machineName: '$machineName'"
    prompt_or_exit
    remapCapsLockKey
    prompt_or_exit
    installMiscSoftwareFromApt
    prompt_or_exit
    installMiscSoftwareFromSnap
    prompt_or_exit
    installMiscSoftwareFromFlatpak
    prompt_or_exit
    installGoogleChrome
    prompt_or_exit
    installGnomeChromeIntegration
    prompt_or_exit
    installBrew
    prompt_or_exit
    installGithubLFS
    prompt_or_exit
    installSshServer
    prompt_or_exit
    installSublimeTextAndMerge
    prompt_or_exit
    installRustup
    prompt_or_exit
end

function installMiscSoftwareFromFlatpak
    flatpak install -y flathub com.jgraph.drawio.desktop
    flatpak install -y org.freedesktop.Platform.GStreamer.gstreamer-vaapi
    flatpak install -y flathub com.visualstudio.code-oss
    flatpak install -y flathub nl.hjdskes.gcolor3
    flatpak install -y flathub org.nickvision.tubeconverter
    flatpak install -y flathub com.github.PintaProject.Pinta
    flatpak install -y flathub io.github.nokse22.asciidraw
    flatpak install -y flathub org.nickvision.tubeconverter
    flatpak install -y flathub com.obsproject.Studio
    flatpak install -y flathub com.ozmartians.VidCutter
    flatpak install -y flathub io.mpv.Mpv
    flatpak install -y flathub nl.hjdskes.gcolor3
    flatpak install -y flathub org.mozilla.firefox
    flatpak install -y flathub org.gnome.Weather
end

# More info: https://www.rust-lang.org/tools/install
function installRustup
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    # The following is needed for anything below (eg cargo, rustup) to work.
    bass source ~/.cargo/env

    # https://github.com/alacritty/alacritty/blob/master/INSTALL.md
    sudo apt install -y cmake pkg-config libfreetype6-dev libfontconfig1-dev \
        libxcb-xfixes0-dev libxkbcommon-dev python3 \
        libssl-dev

    # # https://docs.rs/openssl/latest/openssl/index.html#building
    cargo install --locked cargo-outdated --force --features vendored-openssl

    cargo install alacritty cargo-limit cargo-watch cargo-expand cargo-edit flamegraph bacon exa

    # enables `cargo install-update -a` to update all binaries installed using cargo.
    cargo install cargo-update starship exa fd-find ripgrep lsd r3bl-cmdr cargo-cache nu

    # https://rust-lang.github.io/rustup/concepts/channels.html
    rustup install nightly stable
    rustup component add rust-analyzer

    # https://github.com/helix-editor/helix/wiki/How-to-install-the-default-language-servers
    sudo ln -s $(rustup which rust-analyzer ) /usr/local/bin/rust-analyzer
    rustup toolchain install nightly
    rustup default nightly

    wget https://github.com/alacritty/alacritty/releases/download/v0.9.0/Alacritty.desktop -O ~/.local/share/applications/Alacritty.desktop
end

function remapCapsLockKey
    echo "Remapping caps lock key"
    # https://opensource.com/article/21/5/remap-caps-lock-key-linux
    dconf write /org/gnome/desktop/input-sources/xkb-options "['caps:ctrl_modifier']"
    # dconf write /org/gnome/desktop/input-sources/xkb-options "['caps:ctrl']"
end

function installSublimeTextAndMerge
    printHeader "Install sublime-text and sublime-merge"
    executeString "wget -qO - https://download.sublimetext.com/sublimehq-pub.gpg | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/sublimehq-archive.gpg > /dev/null"
    aptInstall apt-transport-https
    writeAptSourceListFileAsRoot \
        "deb https://download.sublimetext.com/ apt/stable/" \
        sublime-text.list
    aptUpdate
    aptInstall sublime-text sublime-merge
end

# More info on installing Chrome from terminal: https://itsfoss.com/install-chrome-ubuntu/
function installGoogleChrome
    printHeader "Install latest version of Google Chrome"
    # Install google-chrome.
    pushd $HOME/Downloads
    executeString "wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb"
    executeString "wget https://dl.google.com/linux/direct/google-chrome-beta_current_amd64.deb"
    executeString "wget https://dl.google.com/linux/direct/google-chrome-unstable_current_amd64.deb"
    executeString "sudo dpkg -i google-chrome-stable_current_amd64.deb"
    executeString "sudo dpkg -i google-chrome-beta_current_amd64.deb"
    executeString "sudo dpkg -i google-chrome-unstable_current_amd64.deb"
    executeString "sudo apt --fix-broken install"
    popd
end

function installSshServer
    printHeader "Setup SSH server"

    # Enable ssh into this machine.
    aptInstall openssh-server
    executeString "sudo ufw allow ssh"

    set linesToAdd "TCPKeepAlive yes" "ClientAliveInterval 60" "ClientAliveCountMax 120"
    for line in $linesToAdd
        set command "echo '$line' | sudo tee -a /etc/ssh/sshd_config"
        sh -c "$command"
    end
end

# Install git-lfs. More info: https://git-lfs.github.com/
function installGithubLFS
    printHeader "Install GitHUB LFS"

    executeString "curl -s https://packagecloud.io/install/repositories/github/git-lfs/script.deb.sh | sudo bash"
    aptInstall git-lfs
    executeString "git lfs install"
    executeString "sudo git lfs install --system"
end

# More info: https://stackoverflow.com/a/48583923/2085356
function installBrew
    printHeader "Install linuxbrew"
    executeString "curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh | bash"
    executeString "/home/linuxbrew/.linuxbrew/bin/brew install nano cloc gdu micro lsd cmatrix deno pandoc btop inotify-tools"
    executeString "/home/linuxbrew/.linuxbrew/bin/brew install tmux fd ripgrep openssl@1.1 autojump rm-improved bat starship helix node prettier gh"

    # Install prettier, doctoc, etc
    set -gx PATH /home/linuxbrew/.linuxbrew/bin $PATH
    executeString "npm i -g doctoc ts-node ts-node-dev typescript serve"
    ./fix-gnome-session-path-env-var-linuxbrew.fish
    ./fix-usr-local-bin-symlinks.fish
end

function installGnomeChromeIntegration
    printHeader "Install gnome shell & tweaks chrome integration"

    # Gnome extensions and tweaks.
    # More info:https://wiki.gnome.org/Projects/GnomeShellIntegrationForChrome/Installation#Debian_Linux
    executeString "sudo apt install -y chrome-gnome-shell"
end

function installMiscSoftwareFromApt
    printHeader "Install misc software from apt"

    # FS support needed for EXT_SSD.
    aptInstall exfat-fuse

    # Needed for system monitor: https://github.com/paradoxxxzero/gnome-shell-system-monitor-applet.
    aptInstall gir1.2-gtop-2.0 gir1.2-nm-1.0 gir1.2-clutter-1.0 lm-sensors libfuse2

    # Essential packages.
    aptInstall ffmpeg build-essential imagemagick xclip git curl wget font-manager gcc g++ make
    aptInstall openssh-server gnome-tweaks dconf-editor trickle tree nethogs mdns-scan
    aptInstall cmake libevdev-dev libudev-dev libconfig++-dev
    aptInstall tilix guake fontforge gpick clamav pigz pv tree htop
    aptInstall ruby-dev ruby-bundler baobab synaptic lolcat trash-cli
    aptInstall ranger
    aptInstall jq
    aptInstall simplescreenrecorder
    # aptInstall tlp

    # openssl@1.1
    # https://docs.rs/openssl/latest/openssl/index.html#building
    aptInstall pkg-config libssl-dev

    # NFS support.
    aptInstall nfs-common nfs-kernel-server autofs

    # Desktop UI control and automation.
    aptInstall wmctrl xdotool

    # Webcam configuration utils.
    # aptInstall guvcview v4l-utils

    # iotop.
    aptInstall iotop

    # cool retro term, terminator, etc
    aptInstall cool-retro-term terminator

    # Install timeshift: https://github.com/teejee2008/timeshift
    sudo add-apt-repository -y ppa:teejee2008/timeshift
    sudo apt update -y
    sudo apt install -y timeshift

    # Build tools for node-gyp
    sudo apt install -y libx11-dev libxkbfile-dev libsecret-1-dev libfontconfig-dev

    # speedtest cli
    sudo apt-get install -y curl
    curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | sudo bash
    sudo apt-get install -y speedtest

    # linux perf tools (for cargo flamegraph)
    sudo apt install -y linux-tools-common linux-tools-generic

    # install ulauncher from https://omakub.org/
    sudo add-apt-repository universe -y && sudo add-apt-repository ppa:agornostal/ulauncher -y && sudo apt update && sudo apt install ulauncher
end

function installMiscSoftwareFromSnap
    printHeader "Install misc software from snap"
    aptInstall snapd
    snapInstall snippetpixie --classic
    snapInstall vlc
end

# Run the script.
main
