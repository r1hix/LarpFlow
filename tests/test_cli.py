"""Unit tests for CLI command helpers and argument parsing."""
import pytest
from larpflow.core import pipeline


def test_parse_run_args_single():
    repo, prob, sol = pipeline.parse_run_args("my-awesome-repo")
    assert repo == "my-awesome-repo"
    assert prob == ""
    assert sol == ""


def test_parse_run_args_triplet():
    repo, prob, sol = pipeline.parse_run_args("repo-name | High memory leak | Used custom pool allocator")
    assert repo == "repo-name"
    assert prob == "High memory leak"
    assert sol == "Used custom pool allocator"


def test_parse_run_args_invalid():
    with pytest.raises(ValueError):
        pipeline.parse_run_args("repo-name | Only problem")

    with pytest.raises(ValueError):
        pipeline.parse_run_args("")
