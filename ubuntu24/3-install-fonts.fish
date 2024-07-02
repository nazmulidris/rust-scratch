# Victor Mono direct from author.
# ```sh
# fc-list "Victor Mono"
# ```
# Download the latest Victor Mono font from github called `Victor Mono`.
# More info on Iosevka: <https://github.com/rubjo/victor-mono/releases>
# More info on API: <https://api.github.com/repos/rubjo/victor-mono/releases/latest>
# More info on jq: <https://www.baeldung.com/linux/jq-command-json>
# More info on sed: <https://stackoverflow.com/questions/13210880/replace-one-substring-for-another-string-in-shell-script>
function victor-mono-font-upgrade
    pushd ~/Downloads

    # Eg: https://api.github.com/repos/rubjo/victor-mono/releases/latest
    set -l org rubjo
    set -l repo victor-mono
    set -l url "https://api.github.com/repos/$org/$repo/releases/latest"
    # Eg: v26.0.1
    set -l tag_name (curl $url -s | jq .tag_name -r)
    # Eg: 26.0.1
    set -l tag_name (echo $tag_name | sed "s/v//")

    set -l download_file "victor-mono-$tag_name.zip"
    # Eg: https://github.com/rubjo/victor-mono/raw/v1.5.6/public/VictorMonoAll.zip
    set -l download_url "https://github.com/$org/$repo/raw/v$tag_name/public/VictorMonoAll.zip"

    echo (set_color blue)"download_file: " $download_file(set_color normal)
    echo (set_color blue)"download_url: " $download_url(set_color normal)

    sh -c "wget -O $download_file $download_url"
    ls $download_file

    set -l destination_folder ~/.local/share/fonts/VictorMono/
    mkdir -p $destination_folder

    set -l staging_folder ./victor-mono-staging
    mkdir -p $staging_folder

    # Extract the zip file to the staging folder.
    sh -c "unzip -o $download_file -d $staging_folder"
    # Grab all the ttf files from the TTF folder and move them to $destination_folder
    echo (set_color green -o)Moving files from "$staging_folder/TTF/" to "$destination_folder"(set_color normal)
    sh -c "mv $staging_folder/TTF/*ttf $destination_folder"
    # Reset font cache.
    echo (set_color blue)Reset font cache(set_color normal)
    sudo fc-cache -f -v >/dev/null

    # Clean up.
    if test -d $staging_folder
        trash -r $staging_folder
    end
    trash $download_file

    popd
end

# Julia Mono font
# ```sh
# fc-list "Julia Mono"
# ```
# Download the latest font direct from github.com.
function juliamono-font-upgrade
    pushd ~/Downloads
    set -l LATEST_RELEASE_FILE_URL "https://github.com/cormullion/juliamono/releases/latest/download/JuliaMono-ttf.zip"
    curl -OL $LATEST_RELEASE_FILE_URL
    set -l FONT_FILE "JuliaMono-ttf.zip"
    set -l DESTINATION_FOLDER ~/.local/share/fonts/JuliaMono/
    mkdir -p $DESTINATION_FOLDER

    echo (set_color blue)"download_file: " $FONT_FILE(set_color normal)
    echo (set_color blue)"download_url: " $LATEST_RELEASE_FILE_URL(set_color normal)

    echo (set_color green -o)Unzipping $FONT_FILE to $DESTINATION_FOLDER(set_color normal)
    sh -c "unzip -o $FONT_FILE -d $DESTINATION_FOLDER"

    # Reset font cache.
    echo (set_color blue)Reset font cache(set_color normal)
    sudo fc-cache -f -v >/dev/null

    # Clean up.
    trash $FONT_FILE
    popd
end

# Nerd font variant of Iosevka Term called `IosevkaTerm Nerd Font`.
# ```sh
# fc-list "IosevkaTerm Nerd Font"
# ```
# Download the latest patched nerd font for Iosevka called `IosevkaTerm Nerd Font`.
# More info: <https://github.com/ryanoasis/nerd-fonts#option-3-install-script>
function iosevka-nerd-font-upgrade
    pushd ~/Downloads
    set -l LATEST_RELEASE_FILE_URL "https://github.com/ryanoasis/nerd-fonts/releases/latest/download/IosevkaTerm.zip"
    curl -OL $LATEST_RELEASE_FILE_URL
    set -l FONT_FILE "IosevkaTerm.zip"
    set -l DESTINATION_FOLDER ~/.local/share/fonts/iosevka_nerd_font/
    mkdir -p $DESTINATION_FOLDER

    echo (set_color blue)"download_file: " $FONT_FILE(set_color normal)
    echo (set_color blue)"download_url: " $LATEST_RELEASE_FILE_URL(set_color normal)

    echo (set_color green -o)Unzipping $FONT_FILE to $DESTINATION_FOLDER(set_color normal)
    sh -c "unzip -o $FONT_FILE -d $DESTINATION_FOLDER"

    # Reset font cache.
    echo (set_color blue)Reset font cache(set_color normal)
    sudo fc-cache -f -v >/dev/null

    # Clean up.
    trash $FONT_FILE
    popd
end

# Not Nerd font variant of Iosevka Term called `Iosevka Term` (direct from author).
# ```sh
# fc-list "Iosevka Term"
# ```
# Download the latest Iosevka font from github called `Iosevka Term`.
# More info on Iosevka: <https://github.com/be5invis/Iosevka/blob/main/doc/PACKAGE-LIST.md>
# More info on API: <https://api.github.com/repos/be5invis/Iosevka/releases/latest>
# More info on jq: <https://www.baeldung.com/linux/jq-command-json>
# More info on sed: <https://stackoverflow.com/questions/13210880/replace-one-substring-for-another-string-in-shell-script>
function iosevka-font-upgrade
    pushd ~/Downloads

    # set -l LATEST_EXP https://github.com/be5invis/Iosevka/releases/latest
    set -l org be5invis
    set -l repo Iosevka
    set -l url "https://api.github.com/repos/$org/$repo/releases/latest"
    # Eg: v26.0.1
    set -l tag_name (curl $url -s | jq .tag_name -r)
    # Eg: 26.0.1
    set -l tag_name (echo $tag_name | sed "s/v//")

    set -l download_file "PkgTTF-IosevkaTerm-$tag_name.zip"
    set -l download_url "https://github.com/$org/$repo/releases/download/v$tag_name/$download_file"

    echo (set_color blue)"download_file: " $download_file(set_color normal)
    echo (set_color blue)"download_url: " $download_url(set_color normal)

    curl -OL $download_url
    ls $download_file

    set -l destination_folder ~/.local/share/fonts/ttf_iosevka_term/
    mkdir -p $destination_folder

    echo (set_color green -o)Unzipping $download_file to $destination_folder(set_color normal)
    sh -c "unzip -o $download_file -d $destination_folder"

    # Reset font cache.
    echo (set_color blue)Reset font cache(set_color normal)
    sudo fc-cache -f -v >/dev/null

    # Clean up.
    trash $download_file

    popd
end

victor-mono-font-upgrade
juliamono-font-upgrade
iosevka-nerd-font-upgrade
iosevka-font-upgrade

sudo fc-cache -f -v >/dev/null

mkdir -p ~/.config/fontconfig/
cp fonts.conf ~/.config/fontconfig/
