![Crates.io Version](https://img.shields.io/crates/v/ruff-tools)
[![CI](https://github.com/purajit/ruff-tools/actions/workflows/ci.yaml/badge.svg)](https://github.com/purajit/ruff-tools/actions/workflows/ci.yaml)
![GitHub License](https://img.shields.io/github/license/purajit/ruff-tools)

# Installation
Either download release artifacts or

``` sh
# asdf
asdf plugin add ruff-tools https://github.com/purajit/asdf-ruff-tools.git
asdf install ruff-tools 0.1.0

# cargo
cargo install ruff-tools
```

# Usage

``` sh
ruff-tools help
```

## `live`
This will run `ruff-tools` in a loop, while it detects changes in your repo. Whenever
a file is changed, it will run a specified command on all affected files, including
transitive dependencies. For instance, this could be used to automatically run _all_
tests impacted by a change, no matter how far removed it is.

``` sh
ruff-tools live --paths <files/folders to narrow affected files> -- pytest
```

Example run while changing the file `src/util/bar.py`:

``` sh
$ ruff-tools live --paths src/util -- pytest
Constructing initial graph ...
Listening! Ctrl-C to quit.
Changed paths: src/util/bar.py
Transitively affected files: src/util/tests/test_foo.py, src/util/tests/test_bar.py, src/util/baz.py, src/util/bar.py

RUNNING COMMAND!

========================================== test session starts ===========================================
platform darwin -- Python 3.12.7, pytest-8.3.3, pluggy-1.5.0
collected 22 items

src/util/tests/test_foo.py ...............
src/util/tests/test_bar.py .......

-------------- generated xml file: pytest-report.xml ---------------
=========================================== 22 passed in 0.71s ===========================================

COMPLETED RUN!

Changed paths: src/util/bar.py
Transitively affected files: src/util/tests/test_bar.py, src/util/tests/test_foo.py, src/util/bar.py, src/util/foo.py

RUNNING COMMAND!

========================================== test session starts ===========================================
platform darwin -- Python 3.12.7, pytest-8.3.3, pluggy-1.5.0
collected 22 items

src/util/tests/test_bar.py .......
src/util/tests/test_foo.py ...............

-------------- generated xml file: pytest-report.xml ---------------
=========================================== 22 passed in 0.68s ===========================================

COMPLETED RUN!
```

## `detect-cycles`
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

## `minimize-cycles`
You can also pass in the output of `pylint` after removing all your `cyclic-import`
disables, and pass the output to `ruff-tools`, which will minimize the cycles detected
by `pylint` using `ruff`'s graph. This currently only works for projects that use
standard src-layout.

``` sh
ruff-tools minimize-cycles --cycle-results-file <cycle-results-file>
```

## `lint-imports`
Not yet implemented, but will be a drop-in replacement for `import-linter`.
