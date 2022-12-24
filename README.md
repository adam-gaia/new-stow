# New Stow

New Stow (or `nstow`) is a symlink farm manager that aims to superset[^1] GNU Stow (or `stow`).

[^1] See the comparison section below.

## Install

The package `new-stow` provies a binary nammed `nstow`.

### Cargo

```bash
cargo install new-stow
```

### Distro packages

TODO

## Usage

1. Create a stowfile (see Stowfiles section further down)
2. Link files

```bash
nstow --stow
```

3. Unlink files

```bash
nstow --unstow
```

### Examples

- Stow has historically been used to create symlinks from compiled execs to locations on the path.
  See `./examples/exec` for an example
- See `./examples/dotfiles` for an example on using `nstow` to manage dotfiles

### Additional information

`nstow --help`

## Stowfiles

`nstow` searches the working directory for a `stowfile`.
Stowfiles define a set of sources and links.

```yaml
---
vars:
  # Variables may be defined for use in a src or link path
  - THIS_IS_A_VAR=var_value
  # Additionally, environment variables are inherrited

stow:
  - src: some_example_file
    links:
      - ${HOME}/${THIS_IS_A_VAR}/link_it_here # One source file may be linked to many places
      - ${HOME}/some/nested/dir/link_it_here_too # Link's parent directories are created if they do not exist

  - src: alacritty.yml
    links:
      # Example of Stowfile using an env var not defined in the `var` section
      - "${XDG_CONFIG_HOME}/alacritty/alacritty.yaml"

  # Source files may be arbitrarily nested in directories
  - bash:
      - src: bashrc
        links:
          - "${HOME}/.bashrc"

      - src: bash_profile
        links:
          - "${HOME}/.bash_profile"

  # The source can have any name, even something unrelated to the link's name
  - src: readline
    links:
      - "${HOME}/.inputrc"
```

The stowfile above will result in links

- ./some_example_file -> ~/var_value/link_it_here
- ./some_example_file -> ~/some/nested/dir/link_it_here_too
- ./alacritty.yml -> ~/.config/alacritty/alacritty.yml
- ./bash/bashrc -> ~/.bashrc
- ./bash/bash_profile -> ~/.bash_profile
- ./readline -> ~/.inputrc

## Comparison between nstow and gstow

`nstow` aims to superset (most) of `stow`'s features [^2]

| GNU Stow Feature | New Stow | Comments |
| ---------------- | -------- | -------- |
| --no             | ✔        |          |
| --dir            | ✔        |
| --stow           | ✔        |
| --delete         | ✔        |
| --restow         | ✔        |
| --adopt          |          | planned  |
| --no-folding     |          | planned  |
| --ignore=REGEX   | ✔        |
| --defer=REGEX    |          |
| --override=REGEX | ✔        |
| --backup=REGEX   | ✔        |
| --dotfiles       |          | planned  |

Note that stow's regexes may match the beginning or end of a file while nstow regexes match any part.

TODO: do we want full feature parity with stow and the ability to link without a stowfile?
[^2]GNU Stow options are current with `2.3.1`, the latest at the time of writing.

## Developing

## Dependencies

Dependencies are managed with a [Nix Flake](https://nixos.wiki/wiki/Flakes). While we reccomend using Nix, you can ignore it and work with Cargo directly.
The `toolchain` list in `flake.nix` will specify any extra development dependencies.

## Testing

`./run-tests` runs the script `tests/integration-tests` in a container so that we can create/delete symlinks with impunity.
