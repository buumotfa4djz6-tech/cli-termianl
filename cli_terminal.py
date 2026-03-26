#!/usr/bin/env python3
"""
DEPRECATED: This file is deprecated.
Use: python -m cli_terminal instead.
"""

import warnings
warnings.warn(
    "cli_terminal.py is deprecated. Use: python -m cli_terminal",
    DeprecationWarning,
    stacklevel=2
)

# Keep old functionality for backward compatibility
from cli_terminal.app import main

if __name__ == "__main__":
    import sys
    sys.exit(main())
