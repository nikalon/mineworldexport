# Mineworldexport
Small CLI utility to clean and export Minecraft Java Edition worlds. Useful for mapmaking.

## Usage
In Windows:
```sh
mineworldexport DIRECTORY
```

In Linux:
```sh
./mineworldexport DIRECTORY
```

where `DIRECTORY` is a valid root directory for a Minecraft Java Edition world. It will export the world into a new directory called `DIRECTORY_RELEASE`.

## Behavior
Right now this program will clean up the world in the following way:
- Remove old `level.dat` files
- Modify `level.dat` file to disable cheats (`allowCommands` = 1) and remove all player data
- Empty `advancements`, `playerdata` and `stats` directories
- Remove `data/scoreboard.dat` file
