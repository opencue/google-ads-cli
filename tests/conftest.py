"""Pytest config. Loads the gads CLI (single file, no .py extension) as
a module under the name `gads` so tests can import its internals."""

import importlib.machinery
import importlib.util
import pathlib
import sys

import pytest


ROOT = pathlib.Path(__file__).resolve().parent.parent
GADS_PATH = ROOT / "gads"


def _load_gads_module():
    loader = importlib.machinery.SourceFileLoader("gads", str(GADS_PATH))
    spec = importlib.util.spec_from_loader("gads", loader)
    mod = importlib.util.module_from_spec(spec)
    loader.exec_module(mod)
    sys.modules["gads"] = mod
    return mod


_GADS = _load_gads_module()


@pytest.fixture
def gads():
    """Returns the loaded gads module so tests can access its internals."""
    return _GADS
