<div align="center">
    <img src="assets/amber.png" alt="amber logo" width="250" />
</div>

# Amber

Programming language that compiles to Bash. It's a high level programming language that makes it easy to create shell scripts. It's particularly well suited for cloud services.

> [!Warning]
> This software is not ready for extended usage.

[Join our Discord!](https://discord.com/invite/cjHjxbsDvZ)

## Install
Amber compiler currently works on:
- Linux x86 and ARM
- macOS x86 and ARM (Apple Silicon)

### macOS / Linux
Make sure that the operating system meets the following prerequisites
- Bourne-again shell (Bash)
- Curl tool for downloading the installation script
- Basic calculator `bc` command (On Debian run `sudo apt install bc`)

#### system-wide install
```bash
curl -s "https://raw.githubusercontent.com/Ph0enixKM/AmberNative/master/setup/install.sh" | /bin/bash
```

#### local-user install
```bash
curl -s "https://raw.githubusercontent.com/Ph0enixKM/AmberNative/master/setup/install.sh" | /bin/bash -s -- --user
```

#### Via a package manager
Amber is packaged in the following distros:

Arch (AUR) - `amber-bash-bin`

### Windows support
As windows does not come with bash installed it makes no sense to support it. Please install WSL 2 on your windows machine and install Linux version of Amber compiler inside.

In order for it to work you may need to run the following code that pulls all the prerequisites.

```bash
sudo apt install curl bc
sudo mkdir /opt /usr/local/bin
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

Debugging Amber:
```bash
// Shows the AST
AMBER_DEBUG_PARSER=true cargo run <file.ab>
// Shows the time it took to compile each phase
AMBER_DEBUG_TIME=true cargo run <file.ab>

// Flamegraph is a profiling tool that is used to visualize the time each function took to execute
sudo cargo flamegraph -- <file.ab> <file.sh>
```
