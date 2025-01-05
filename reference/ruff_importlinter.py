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
from importlinter.application.rendering import render_contract_result_line
from importlinter.domain.contract import registry

logger = logging.getLogger()

CONTRACT_KEY_PREFIX = "importlinter:contract:"

RED = "\033[91m"
GREEN = "\033[92m"
NC = "\033[0m"

MODULE_REGEX = "[a-zA-Z0-9_]+"
MODULE_PATH_REGEX = rf"({MODULE_REGEX}\.)*{MODULE_REGEX}"


# doesn't fully implement python packaging rules, but works for standard src-layouts
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

        print(f"Checking {contract.name}... ", end="")
        check = contract.check(deepcopy(import_graph), verbose=False)
        num_violating_import = sum(
            len(m["chains"]) for m in check.metadata.get("invalid_chains", [])
        )
        if num_violating_import:
            print(f"{RED}BROKEN{NC}! Found {num_violating_import} violating imports")
            contract.render_broken_contract(check)
        else:
            print(f"{GREEN}KEPT{NC}!")


def _ignore_imports_to_regex(ignore_rules: list[str]) -> list[str]:
    return [
        ignore_rule.replace("**", MODULE_PATH_REGEX).replace("*", MODULE_REGEX)
        for ignore_rule in ignore_rules
    ]


def main() -> int:
    print("Loading configuration...", end="")
    from importlinter.configuration import configure

    configure()
    print(f"{GREEN}DONE!{NC}")
    print("Building import graph...", end="")
    dependency_graph = {
        path_to_module(k): set({path_to_module(m) for m in v})
        for k, v in json.loads(
            subprocess.check_output(["ruff", "analyze", "graph", "--preview"])
        ).items()
    }
    print(f"{GREEN}DONE!{NC}")

    import_graph = ImportGraph()

    for module, deps in dependency_graph.items():
        import_graph.add_module(module)
        for dep in deps:
            import_graph.add_module(dep)
            import_graph.add_import(
                importer=module,
                imported=dep,
                line_number=1,
                line_contents="placeholder",
            )

    _run_checks(import_graph)


if __name__ == "__main__":
    exit(main())
