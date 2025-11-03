"""
Inspectra - Memory Analysis and Manipulation Framework
Python bindings for the Inspectra core engine.
"""

from .inspectra import (
    ProcessManager,
    ProcessInfo,
    Scanner,
    version,
    init,
)

__version__ = version()
__all__ = [
    'ProcessManager',
    'ProcessInfo',
    'Scanner',
    'version',
    'init',
]

# Initialize on import
init()
