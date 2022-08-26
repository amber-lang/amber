<div align="center">
    <img src="assets/amber.png" alt="amber logo" width="250" />
</div>

# Amber

Programming language that compiles to Bash. It's a high level programming language that makes it easy to create shell scripts. In addition to that - the main functionality works in an abstraction layer meaning that you can recompile standard library in a way so that it uses your proxy that can disable / enable certain features. It's particulary well suited for cloud services.

> **[Warning]**
> This software is not ready for extended usage.

## Contributing
In order to contribute, you have to add couple of build targets:
```bash
rustup target add aarch64-unknown-linux-gnu
rustup target add aarch64-apple-darwin
rustup target add aarch64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add x86_64-pc-windows-gnu
```

And linkers (macos):
```bash
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
brew install mingw-w64
```