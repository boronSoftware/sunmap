# sunmap

I was dissatisfied with the existing solutions for extracting sourcemaps.
They kept cluttering my system by naively joining paths with multiple level-up notations, extracting things to parent directories.

This tool will handle such cases by analyzing all paths, and then creating a base directory such that nothing gets extracted outside of the specified output directory.

## Usage
```sh
sunmap --source-map path/to/sourcemap.js.map --out-dir ./project
# or
sunmap --source-map-dir directory/of/sourcemaps --recursive --out-dir ./project
```
