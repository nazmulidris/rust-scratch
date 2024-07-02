#!/usr/bin/env fish

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
