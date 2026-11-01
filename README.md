# markers

Highlight TODO-style markers in the current Kawi buffer, add gutter signs, and show a marker count in the status line.

## What it does

The plugin scans the current buffer for these uppercase markers:

- `TODO`
- `FIXME`
- `HACK`
- `NOTE`
- `XXX`

Each marker gets a padded badge, and the rest of the comment is shown in the same color with italics. Lines with markers get one gutter sign. The status line shows the total marker count while the buffer has at least one marker.

The plugin adds one command:

```text
:markers.scan
```

Use it to refresh the display after changing settings or to show a summary message.

## Install

If this plugin is published at `github.com/OWNER/kawi-markers-plugin`, install it with:

```text
:plugin install OWNER/kawi-markers-plugin
```

To install a specific release:

```text
:plugin install OWNER/kawi-markers-plugin@v0.1.2
```

Replace `OWNER` with the GitHub account that owns the repository.

## Usage

1. Open a file in Kawi.
2. Add a marker such as `TODO: handle empty input`.
3. The marker is highlighted, a gutter sign appears, and the status line shows the total count.
4. Run `:markers.scan` to refresh and show a summary message.

Marker matching is case-sensitive and requires word boundaries. `TODO` matches, while `TODOLIST` and `todo` do not.

## Settings

The plugin declares two settings:

```toml
[markers]
show_signs = true
show_virtual_text = true
```

Use `:plugin settings markers` to edit them. Changes apply on the next scan or buffer event.

## Host APIs

- `editor.read`
- `editor.decorations`
- `ui.message`
- `ui.statusline`

The plugin does not write to the buffer, read files, spawn processes, or use the network.

## Manage this plugin

```text
:plugin update markers
:plugin disable markers
:plugin enable markers
:plugin remove markers
:plugin logs markers
```

## Compatibility

- Plugin API: 1
- Runtime: WebAssembly
- SDK: `kawi-plugin-sdk` 0.5.0

For local development and release steps, see [DEVELOPING.md](DEVELOPING.md).
