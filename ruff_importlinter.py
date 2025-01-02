#! .venv/bin/python
import json
import logging
import re
import subprocess
from copy import deepcopy
from typing import TypedDict

from grimp.adaptors.graph import ImportGraph
from importlinter.application.use_cases import (
    read_user_options,
    _filter_contract_options,
    _register_contract_types,
)
from importlinter.domain.contract import registry

logger = logging.getLogger()

CONTRACT_KEY_PREFIX = "importlinter:contract:"

MODULE_REGEX = "[a-zA-Z0-9_]+"
MODULE_PATH_REGEX = rf"({MODULE_REGEX}\.)*{MODULE_REGEX}"


class Violation(TypedDict):
    module_name: str


class Layer(TypedDict):
    modules: list[str]
    allow_intra_layer_imports: bool


# doesn't fully implement python packaging rules, but works for what we have
def path_to_module(path: str) -> str:
    path = path.replace("/__init__.py", "").replace("/", ".")
    path = re.sub(r"\.py$", "", path)
    if "src." in path:
        start_index = path.index("src.") + len("src.")
        path = path[start_index:]
    return path


def _run_checks(import_graph) -> None:
    user_options = read_user_options(".importlinter")
    _register_contract_types(user_options)
    contracts_options = _filter_contract_options(user_options.contracts_options, ())
    for contract_options in contracts_options:
        contract_class = registry.get_contract_class(contract_options["type"])
        contract = contract_class(
            name=contract_options["name"],
            session_options=user_options.session_options,
            contract_options=contract_options,
        )

        print(f"Checking {contract.name}...")
        check = contract.check(deepcopy(import_graph), verbose=False)
        print(sum(len(m["chains"]) for m in check.metadata.get("invalid_chains", [])))


def _ignore_imports_to_regex(ignore_rules: list[str]) -> list[str]:
    return [
        ignore_rule.replace("**", MODULE_PATH_REGEX).replace("*", MODULE_REGEX)
        for ignore_rule in ignore_rules
    ]


def main() -> int:
    from importlinter.configuration import configure

    configure()
    dependency_graph = {
        path_to_module(k): set({path_to_module(m) for m in v})
        for k, v in json.loads(
            subprocess.check_output(["ruff", "analyze", "graph", "--preview"])
        ).items()
    }

    import_graph = ImportGraph()

    for module, deps in dependency_graph.items():
        import_graph.add_module(module)
        for dep in deps:
            import_graph.add_module(dep)
            import_graph.add_import(importer=module, imported=dep)
    _run_checks(import_graph)


if __name__ == "__main__":
    exit(main())
