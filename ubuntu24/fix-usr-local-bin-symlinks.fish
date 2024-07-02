#!/usr/bin/env fish

pushd /usr/local/bin/
sudo ln -s (which prettier)
sudo ln -s (which xdg-open) open
sudo ln -s (which node)
sudo ln -s (which doctoc)
sudo ln -s (which ts-node)
sudo ln -s (which ts-node-dev)
popd
