<div align="center">
    <img src="assets/amber.png" alt="amber logo" width="250" />
</div>

# Amber

Programming language that compiles to Bash. It's a high level programming language that makes it easy to create shell scripts. In addition to that - the main functionality works in an abstraction layer meaning that you can recompile standard library in a way so that it uses your proxy that can disable / enable certain features. It's particulary well suited for cloud services.

> **[Warning]**
> This software is not ready for extended usage.

## Install
This compiler currently works on Windows (WSL), Linux and MacoOS - all x86 and ARM 64 bit.

### MacOS
Make sure that your operating system satisfies the follorwing prerequsites
- Bash or Zsh or any other Bourne-again shell (usually comes with MacOS)
- Ruby 2.0 or newer (usually comes with MacOS)

```bash
sudo ruby -e "require 'open-uri'; puts open('https://raw.githubusercontent.com/Ph0enixKM/AmberNative/master/setup/install.sh').read" | $(echo $SHELL)
```

### Linux
Make sure that your operating system satisfies the follorwing prerequsites
- Bash or Zsh or any other Bourne-again shell
- Curl tool for downloading the installation script

```bash
sudo curl https://raw.githubusercontent.com/Ph0enixKM/AmberNative/master/setup/install.sh | $(echo $SHELL)
```


## Contributing
In order to contribute, you have to add couple of build targets:
```bash
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-apple-darwin
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-apple-darwin
```

And linkers (macos):
```bash
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-musl
brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu
```

Finally in order to build
```bash
amber build.ab
```

In order to parse AST with a debug trace run cargo with the following environment variable:
```bash
AMBER_DEBUG_PARSER=true cargo run <file.ab>
```
