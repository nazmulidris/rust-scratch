# ubuntu24

These scripts are for setting up a new Ubuntu 24.04 desktop machine. They have been tested
on a fresh install of Ubuntu 24.04 LTS. They contain all the software that is needed for
Rust development, OBS Studio use, and general computing.

Lots of customized font configurations are included in the scripts. Once you clone this
repo, the scripts can be run in the following order:

1. `0-bootstrap.bash`
2. `1-install.fish`
3. `2-install-docker.bash`
4. `3-install-fonts.fish`

Note that it's not possible to use `curl -s https://raw.githubusercontent.com/...` to run
these scripts, since they have dependencies on other scripts like `utils.fish`, etc.