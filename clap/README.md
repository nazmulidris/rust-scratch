# clap

This CLI app is built using this excellent tutorial:
<https://rust-cli.github.io/book/in-depth/machine-communication.html#how-to-deal-with-input-piped-into-us>.

In order to run this app and have it read from stdin, you need to do the following:

```bash
echo -e "foo\nbar" | cargo run -- edit foo.txt -s -j # output is in json
echo -e "foo\nbar" | cargo run -- edit foo.txt -s # output is human readable text
```

Here's more information on how to use the derive version of the clap crate:
<https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#quick-start>
