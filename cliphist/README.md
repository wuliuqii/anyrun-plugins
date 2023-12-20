# Cliphist

Find clipboard history using the [cliphist](https://github.com/sentriz/cliphist).

## Usage

Type in `<prefix><word to find>`, where prefix is the configured prefix (default in Configuration).

![cliphist](https://github.com/wuliuqii/anyrun-plugins/assets/34090258/eefe24c1-1ee9-4128-83d8-d7282b397095)

## Dependence

- [cliphist](https://github.com/sentriz/cliphist)
- [wl-clipboard (for wayland)](https://github.com/bugaevc/wl-clipboard)

## Configuration

```ron
// <anyrun config dir>/cliphist.ron
Config(
  cliphist_path: "cliphist",
  max_entries: 10, 
  prefix: ":v",
)
```

## TODO

- [ ] Image support

