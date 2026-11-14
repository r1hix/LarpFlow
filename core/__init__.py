"""Backward compatibility shim for larpflow.core."""
from larpflow.core import config, db, drafts, pipeline

__all__ = ["config", "db", "drafts", "pipeline"]
