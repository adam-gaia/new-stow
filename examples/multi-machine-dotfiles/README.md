# Nested dotfile example

- Machines A & B
  - have the same alacritty.yml
  - each have their own sway/config
- Machine C
  - has its own alacritty.yml
  - has its own sway/config

## Stowing for Machine A

`nstow --dir machineA` or `cd machineA && nstow`

- Resulting symlinks
  - ~/.config/alacritty/alacritty.yml -> <this-example-dir>/**commonAB**/alacritty/alacritty.yml
  - ~/.config/sway/config -> <this-example-dir>/**machineA**/sway/config

## Stowing for Machine B

`nstow --dir machineB` or `cd machineB && nstow`

- Resulting symlinks
  - ~/.config/alacritty/alacritty.yml -> <this-example-dir>/**commonAB**/alacritty/alacritty.yml
  - ~/.config/sway/config -> <this-example-dir>/**machineB**/sway/config

## Stowing for Machine C

`nstow --dir machineC` or `cd machineC && nstow`

- Resulting symlink
  - ~/.config/alacritty/alacritty.yml -> <this-example-dir>/**machineC**/alacritty/alacritty.yml
  - ~/.config/sway/config -> <this-example-dir>/**machineC**/sway/config
