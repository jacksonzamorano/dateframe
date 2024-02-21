# Dateframe

Clean up folders & files!

## About Dateframe
Dateframe is meant to keep a running collection of folders. The reason I created it is I copy security camera footage from my security system to my NAS, but I don't want to keep copies for all time. This will allow me to scrub extraneous footage automatically.

## How to use

### From Docker (recommended):
1. Create a `dateframe.conf` in the root of the folder you want to maintain (see documentation below).
2. Run the following Docker command, replacing the example path with your path:
`docker run -v /your/path/here:/var/data -d --name dateframe ghcr.io/jacksonzamorano/dateframe:latest`

### From the commmand line:
1. Create a `dateframe.conf` in the root of the folder you want to maintain (see documentation below).
2. Install the binary using `cargo install dateframe`.
3. Run `dateframe path/to/your/directory`.

## Optional command line params
- `--gen-test`: Generates a structure of test files.
- `--once`: Runs once, and does not start a daemon.

## Configuration documentation
Configuration is provided in the format key=value (without spaces). Note all times are parsed and formatted in your machine's local time.

### `format`
Format specifies how dates from file/foldernames should be parsed. The format is provided by the `chrono` crate, check out the documentation [here](https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers). Multiple formats can be provided, and all will be tried on each file.

### `retention`
Retention specifies how long folders & files should stay. Here are the following formats:
- `xxd` (replacing xx with number of days)
- More to come soon

### `deep`
Defaults to `true`. Can be `true` or `false`. If `true`, folders are recursively searched instead of just cleaning the root directory.

### `refresh`
Number specified in seconds. After cleaning, this number specifies how long dateframe should sleep before cleaning again. Default is 360 (1 hour).

### `log`
Defines how much information will go to the console. They are listed in most verbose to least:
- `debug`
- `info`
- `error`
- `silent`

Default is info.

### `remove`
Specific words/items to be removed. Multiple copies of this key can be provided and all will be used.

### `split_string`
For more complicated filenames, you may want to split on a specific string or character. For example, consider this filename: "2024-02-21T04-23-57 smartDetectZone (person)". We could add "smartDetectZone" and "(person)" to the `remove` key but it would instead me easier to split on a string and use the first element.

So, set `split_string` to an empty space `split_string= `.

### `split_index`
After a `split_string` is provided and executed, select which indicies to be used. Multiple copies of this key can be provided and all will be used.

### `split_join`
After a `split_string` is performed and the proper selections are found, the selected strings are joined using this value. Default is "" (blank string).

### `date_only_behavior`
If only a date format is used (for example YYYY-mm-DD), a time needs to be added to determine if the date is past the retention policy. There are several options:
- `start`: 00:00:00
- `noon`: 12:00:00
- `end`: 23:59:59
- `h[x]`: x:00:00

The default is `start`.