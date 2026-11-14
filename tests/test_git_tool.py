"""Unit tests for local Git repository inspection and path resolution."""
from larpflow.tools import tool_git


def test_resolve_repo_path_current_dir():
    resolved = tool_git.resolve_repo_path(".")
    assert resolved is not None
    assert (resolved / ".git").exists()


def test_resolve_repo_path_nonexistent():
    assert tool_git.resolve_repo_path("nonexistent_path_xyz_987") is None


def test_is_local_repo():
    assert tool_git.is_local_repo(".") is True
    assert tool_git.is_local_repo("nonexistent_directory_12345") is False
    assert tool_git.is_local_repo("") is False
