# NixOS and Flakes

The package contains all the required install scripts and dependencies. You can use the flake as:

```nix
{
    inputs = {
        # ...
        amber.url = "github:Ph0enixKM/Amber";
    };

    # then later with home manager for example
    home.packages = [ inputs.amber.packages.${pkgs.system}.default ];
}
```

The package is available as `amber-lang` on [nixpkgs](https://github.com/NixOS/nixpkgs/pull/313774).

While developing with Nix, the flake defines all dependencies for `nix develop` (or `direnv` if used).
