#!/usr/bin/env bash

# Run this script first and it will setup the computer to be able to: Install fish shell
# which will be used to run `1-install.fish` script which actually does the work.

# Add user to sudoers list so no need to type password anymore.
# `tee -a` appends to the given file.
echo "$USER ALL=(ALL) NOPASSWD:ALL" | sudo tee -a /etc/sudoers

# Change the default lid close behavior. More info:
# https://itsubuntu.com/configure-lid-close-behavior-of-your-laptop-with-ubuntu-20-04-lts/
# `tee -a` appends to the given file.
echo 'HandleLidSwitch=suspend'             | sudo tee -a /etc/systemd/logind.conf
echo 'HandleLidSwitchExternalPower=ignore' | sudo tee -a /etc/systemd/logind.conf
echo 'HandleLidSwitchDocked=ignore'        | sudo tee -a /etc/systemd/logind.conf

# Provide support for exFAT formatted filesystems, etc.
sudo apt update -y
sudo apt install -y exfat-fuse curl wget git gnome-tweaks font-manager pigz make gcc

# Install the latest version of fish v3 (apt has older versions).
sudo apt-add-repository -y ppa:fish-shell/release-3
sudo apt update -y
sudo apt -y install fish

# Setup fish in order to use Bass to run bash scripts.
pushd .
mkdir -p $HOME/github/
cd $HOME/github
git clone https://github.com/edc/bass.git
cd bass
make install
popd

# Install flatpak.
sudo apt -y install flatpak
sudo apt -y install gnome-software-plugin-flatpak
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

# Refresh apt.
# More info - https://embeddedinventor.com/apt-upgrade-vs-full-upgrade-differences-explained-for-beginners/
sudo apt full-upgrade -y
sudo apt autoremove -y

# Make fish the default shell.
echo (which fish) | sudo tee -a /etc/shells
sudo chsh --shell (which fish)

echo "⛔ Please logout and log back in to start using using fish shell, and then run 1-install.fish"
echo "⛔ Please restart for flatpak, and then run 1-install.fish"
