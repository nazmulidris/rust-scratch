# ubuntu24

These scripts are for setting up a new Ubuntu 24.04 desktop machine. They have been tested
on a fresh install of Ubuntu 24.04 LTS. They contain all the software that is needed for
Rust development, OBS Studio use, and general developer productivity.

Here's the accompanying [video for this repo on YouTube](https://youtu.be/zGljNewTc0I).

## What comes with the scripts

Here is a non exhaustive list of software that will be installed:

1. `fish` as the default login shell. All the configuration scripts are written in `fish`.
    `bass` is also installed to allow for running `bash` scripts in `fish`.
2. `rustup`, `brew`, and `flatpak` with `flathub` as package managers.
3. `docker` and `docker-compose` for containerization.
4. `obs-studio` for screen recording and streaming.
5. `vlc`, `mpv` for media playback.
6. `chrome` for web browsing.
7. `vscode` for code editing.
8. Lots of Gnome extensions for desktop customization.
9. `nerd-fonts` for terminal font customization. Along with `guake` and `tilix` for
    terminal emulators. Along with `tmux` for terminal multiplexing.

> To download Ubuntu 24.04, visit the [Ubuntu
> website](https://ubuntu.com/download/desktop) and prepare a USB drive with the ISO file
> for installation. You can use [Popsicle](https://flathub.org/apps/com.system76.Popsicle)
> to create a bootable USB drive.

## Running the scripts

Lots of customized font configurations are included in the scripts. You can clone the repo
and run the scripts, or just copy the links below and run them in your terminal.

The scripts can be run in the following order. Really the only one that is required to be run
first is the `0-bootstrap.bash` script. The rest can be run in any order.

1. `curl -s https://raw.githubusercontent.com/nazmulidris/rust-scratch/main/ubuntu24/0-bootstrap.bash | bash`
2. `curl -s https://raw.githubusercontent.com/nazmulidris/rust-scratch/main/ubuntu24/1-install.fish | fish`
3. `curl -s https://raw.githubusercontent.com/nazmulidris/rust-scratch/main/ubuntu24/2-install-docker.bash | bash`
4. `curl -s https://raw.githubusercontent.com/nazmulidris/rust-scratch/main/ubuntu24/3-install-fonts.fish | fish`

Optional scripts:
1. `curl -s https://github.com/nazmulidris/rust-scratch/blob/main/ubuntu24/install-agent-into-bin.fish | fish`
2. `curl -s https://github.com/nazmulidris/rust-scratch/blob/main/ubuntu24/fix-gnome-session-path-env-var-linuxbrew.fish | fish`
3. `curl -s https://github.com/nazmulidris/rust-scratch/blob/main/ubuntu24/fix-usr-local-bin-symlinks.fish | fish`

## Gnome Extensions

- https://extensions.gnome.org/extension/4548/tactile/
- https://extensions.gnome.org/extension/7065/tiling-shell/
- https://extensions.gnome.org/extension/5660/weather-or-not/
- https://extensions.gnome.org/extension/1460/vitals/
- https://extensions.gnome.org/extension/6242/emoji-copy/
- https://extensions.gnome.org/extension/4839/clipboard-history/
- https://extensions.gnome.org/extension/4679/burn-my-windows/
- https://extensions.gnome.org/extension/3843/just-perfection/

## Keyboard remapping

- https://askubuntu.com/questions/977876/changing-command-super-q
- https://flameshot.org/docs/guide/wayland-help/
- https://github.com/Ulauncher/Ulauncher/wiki/Hotkey-In-Wayland
- https://askubuntu.com/questions/26056/where-are-gnome-keyboard-shortcuts-stored

## Chrome issues w/ Wayland

- https://askubuntu.com/a/1502896/872482
- navigate to `chrome://flags`
- change `Preferred Ozone Platform` from `default` to `wayland`

## libfuse2 and AppImage issues

- [`libfuse2`](https://github.com/AppImage/AppImageKit/wiki/FUSE) is not included with
  Ubuntu 24.04. `AppImage`s are difficult to run (since they need `libfuse2` installed).
- To run them, have to pass an extra flag in the terminal or `.desktop` file(eg for
  `uhk-agent`). here's a workaround (to keep from installing `libfuse2`).
- Here's an example of the command to run the [`uhk-agent`
  AppImage](https://ultimatehackingkeyboard.com/agent):

    ```bash
    /UHK.Agent-4.2.0-linux-x86_64.AppImage --appimage-extract
    cd squashfs-root
    ./uhk-agent --no-sandox
    ```

## Settings -> keymappings

- To create keyboard shortcuts that launch a shell command, wrap it in `sh -c $CMD`. This
  is what must be done for `flameshot`, and `ulauncher`.
- Bind `ulauncher-toggle` to the settings -> keyboard shortcuts in gnome.

## OBS Studio issues

`obs-studio` has some UI issues, and dialog boxes are quite glitchy and don't display
properly. keyboard shortcuts can't be reliably used when the `obs-studio` window is not in
focus. can't really bind to settings -> keyboard shortcuts either, since there's no
command to stop recording; start recording will spawn a new process.

## Tilix and quake mode

- `tilix` and `quake mode` is disabled in `wayland`; had to install `quake`
  - https://lukaszwrobel.pl/blog/tmux-tutorial-split-terminal-windows-easily/
  - use `tmux` to get tiling functionality similar to `tilix` in `guake`.

## Fontconfig

Custom font install using script. optional - `~/.config/fontconfig/fonts.conf` change
for system fonts that affect all apps. also `gnome-tweaks` to change fonts, and other
settings.

- https://jichu4n.com/posts/how-to-set-default-fonts-and-font-aliases-on-linux/
- https://www.freedesktop.org/software/fontconfig/fontconfig-user.html
- https://en.wikipedia.org/wiki/Fontconfig
