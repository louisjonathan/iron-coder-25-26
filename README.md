# iron-coder-25-26
This project serves as a testing environment for future Iron Coder features. While familiarizing ourselves with the codebase, we will utilize new libraries and create features in a simplified and abstract manner.

## Test Application Goals:
Get a working proof of concept for the following:
- Window Docking (Jonathan B)
	- Utilize egui_dock to expand the functionality of egui. Generate tabs that can be opened and closed, each with modularity and different functionality.
- Keybinding Support (Evan P)
	- Create a modular keybindings system that reads egui inputs and does actions based on the hotkey configuration. Parses a .json file for hotkey editing and addition.
- Color Scheme (Jonathan L)
	- Modify egui styling using color schemes.

## Known Bugs
- Tabs opened using the "View" menu appear on the focused window split. Ideally, each tab would have a default location that it would dock to (for example, File Explorer on the left, Terminal at the bottom).

### Works Cited

[Color Palettes](https://github.com/Experience-Monks/nice-color-palettes/tree/master)
