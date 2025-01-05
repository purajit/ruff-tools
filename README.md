Contains a bunch of tools built on top of ruff:

# Installation
When I feel confident, I might publish this to crate.io.

``` sh
cargo install --git https://github.com/purajit/ruff-tools
```

# Usage

``` sh
ruff-tools help
```

## live
This will run `ruff-tools` in a loop, while it detects changes in your repo. Whenever
a file is changed, it will run a specified command on all affected files, including
transitive dependencies. For instance, this could be used to automatically run _all_
tests impacted by a change, no matter how far removed it is.

``` sh
ruff-tools live --paths <files/folders to narrow affected files> -- pytest
```

## detect-cycles
This will not only detect cycles (currently using the same algorithm as pylint,
but stay tuned for improvements), but also _minimize_ and unique-ify them. As an
example, in a very large repo, pylint detected ~2000 cycles with a total of ~94,000
edges with the longest cycle being 100 nodes, while `ruff-tools` reduced that down
to ~100 cycles with a total of ~500 edges, and the longest cycle being 18 in length.
It will also find the most common edges in the cycles, which could indicate places
where you might be able to break the most cyclic dependencies at once.

``` sh
ruff-tools detect-cycles
```

## minimize-cycles
You can also pass in the output of `pylint` after removing all your `cyclic-import`
disables, and pass the output to `ruff-tools`, which will minimize the cycles detected
by `pylint` using `ruff`'s graph. This currently only works for projects that use
standard src-layout.

``` sh
ruff-tools minimize-cycles --cycle-results-file <cycle-results-file>
```

## lint-imports
Not yet implemented, but will be a drop-in replacement for `import-linter`.
