# Dateframe

Clean up folders & files!

## How to use

### From the commmand line:
1. Create a `dateframe.conf` in the root of the folder you want to maintain (see documentation below).
2. Install the binary using `cargo install dateframe`.
3. Run `dateframe path/to/your/directory`.

That's all!

## Configuration documentation
Configuration is provided in the format key=value (without spaces).

### `format`
Format specifies how dates from file/foldernames should be parsed. The format is provided by the `chrono` crate, check out the documentation [here](https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers).

### `retention`
Retention specifies how long folders & files should stay. Here are the following formats:
- `xxd` (replacing xx with number of days)
- More to come soon

### `deep`
Defaults to `true`. Can be `true` or `false`. If `true`, folders are recursively searched instead of just cleaning the root directory.

### `refresh`
Number specified in seconds. After cleaning, this number specifies how long dateframe should sleep before cleaning again. Default is 360 (1 hour).