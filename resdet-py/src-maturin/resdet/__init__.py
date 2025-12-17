"""
resdet
~~~~~~
Detect source resolution of upscaled images

:copyright: (c) 2025-present noaione
:license: MPL 2.0, see LICENSE for more details.
"""

from __future__ import annotations

from ._resdet import (  # type: ignore
    Analysis,
    DetectedResolution,
    DetectionResult,
    __version__,
    lib_version,
    normalize_image_gray,
)

__all__ = (
    "Analysis",
    "DetectedResolution",
    "DetectionResult",
    "__version__",
    "lib_version",
    "normalize_image_gray",
)
