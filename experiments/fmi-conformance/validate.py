#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Validate a minimal FMI 3.0 modelDescription and build a container.

This checks the description/container contract only. It does not load or execute
an FMU, so it cannot establish co-simulation behavior or challenge a survival
conclusion by itself.
"""

from __future__ import annotations

import sys
import tempfile
import zipfile
from pathlib import Path
from xml.etree import ElementTree

ROOT = Path(__file__).resolve().parent
DESCRIPTION = ROOT / "modelDescription.xml"
ALLOWED_CAUSALITY = {"parameter", "calculatedParameter", "input", "output", "local", "independent"}
ALLOWED_VARIABILITY = {"constant", "fixed", "tunable", "discrete", "continuous"}
REQUIRED_VARIABLES = {"Float64", "Float32", "Int64", "Int32", "Int16", "Int8", "UInt64", "UInt32", "UInt16", "UInt8", "Boolean", "String", "Binary", "Clock", "Enumeration"}


def fail(message: str) -> int:
    print(f"fmi-model-description=FAIL {message}", file=sys.stderr)
    return 1


def main() -> int:
    tree = ElementTree.parse(DESCRIPTION)
    root = tree.getroot()
    if root.tag != "fmiModelDescription":
        return fail("root element is not fmiModelDescription")
    if root.get("fmiVersion") != "3.0":
        return fail("fmiVersion is not 3.0")
    if not root.get("modelName"):
        return fail("modelName is missing")
    if not root.get("instantiationToken"):
        return fail("instantiationToken is missing")
    cosim = root.find("CoSimulation")
    if cosim is None or not cosim.get("modelIdentifier"):
        return fail("CoSimulation/modelIdentifier is missing")
    variables = root.find("ModelVariables")
    if variables is None or len(variables) == 0:
        return fail("no model variables")
    references: set[str] = set()
    for variable in variables:
        if variable.tag not in REQUIRED_VARIABLES:
            return fail(f"unsupported variable type {variable.tag}")
        reference = variable.get("valueReference")
        if not reference:
            return fail("variable without valueReference")
        if reference in references:
            return fail(f"duplicate valueReference {reference}")
        references.add(reference)
        if variable.get("causality") not in ALLOWED_CAUSALITY:
            return fail(f"invalid causality for {variable.get('name')}")
        if variable.get("variability") not in ALLOWED_VARIABILITY:
            return fail(f"invalid variability for {variable.get('name')}")
    with tempfile.TemporaryDirectory() as raw:
        fmu = Path(raw) / "poie-minimal.fmu"
        with zipfile.ZipFile(fmu, "w", zipfile.ZIP_DEFLATED) as archive:
            archive.write(DESCRIPTION, "modelDescription.xml")
            archive.writestr("sources/.keep", "")
        with zipfile.ZipFile(fmu) as archive:
            if "modelDescription.xml" not in archive.namelist():
                return fail("container lacks modelDescription.xml")
    print("fmi-version=3.0")
    print("co-simulation=declared")
    print(f"variables={len(references)}")
    print("container=zip")
    print("fmi-model-description=ok")
    print("fmi-cosimulation=not_run")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
