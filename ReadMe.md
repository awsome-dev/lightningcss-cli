## lightningcss cli

Minify the files with .css extension in the input directory (default: public) and save them in the output directory (default: dist).

### Setup

```
cargo build --release
```

### Usage

Case 1: Use the default value.

```
./target/release/lightningcss-cli
```

Case 2: Specify custom input and output directories on the command line.

```
./target/release/lightningcss-cli -i [input dir] -o [output dir]
```

Case 3: Specify custom input and output directories in the Config file.

```
./target/release/lightningcss-cli -c [config file]
```

### Config file format

toml format

```toml
# config.toml
input = "public"
output = "dist"
```
