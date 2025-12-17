"""
resdet._resdet
~~~~~~~~~~~~~~
The `.pyd` file for resdet, contains the internal library written in Rust.

:copyright: (c) 2025-present noaione
:license: MPL 2.0, see LICENSE for more details.
"""

from __future__ import annotations

from typing import Iterable

__version__: str
"""Current version of resdet"""

def lib_version() -> str:
    """
    Get the version of the resdet library.

    Raises
    ------
    RuntimeError
        If the version cannot be determined.
    """
    ...

def normalize_image_gray(image: Iterable[int]) -> list[float]:
    """
    Normalize grayscale image data from u8 to f32 (0.0-1.0).

    Parameters
    ----------
    image: :type:`list[int]`
        Raw image data as u8 values.

    Returns
    -------
    :type:`list[float]`
        Normalized image data as f32 values from 0.0 to 1.0.
    """
    ...

class DetectedResolution:
    """The detected resolution of an image."""

    @property
    def size(self) -> int:
        """
        The detected resolution size.
        """
        ...

    @property
    def confidence(self) -> float:
        """
        The confidence level of the detected resolution.

        If -1.0, it means the confidence could not be determined.
        """
        ...

class DetectionResult:
    """The result of a resolution detection."""

    @property
    def widths(self) -> list[DetectedResolution]:
        """
        List of detected resolutions for widths.
        """
        ...

    @property
    def heights(self) -> list[DetectedResolution]:
        """
        List of detected resolutions for heights.
        """
        ...

    def best_width(self) -> DetectedResolution | None:
        """
        Get the best detected width resolution.
        """
        ...

    def best_height(self) -> DetectedResolution | None:
        """
        Get the best detected height resolution.
        """
        ...

class Analysis:
    """
    Analyze images to detect their source resolution.

    Thread Safety
    -------------
    This class *should* be thread-safe, allowing concurrent analyses
    from multiple threads.

    BUT, when trying to use it in multi-thread context, make sure you
    did not change any parameters while another thread is using it,
    since this would result in a BorrowError.
    """

    def __init__(self) -> None:
        """Initialize a new Analysis instance with default parameters."""
        ...

    @property
    def range(self) -> int:
        """
        The range used for analysis.
        """
        ...

    @range.setter
    def range(self, value: int) -> None:
        """
        Set the range used for analysis.
        """
        ...

    @property
    def threshold(self) -> float | None:
        """
        The threshold used for analysis.
        """
        ...

    @threshold.setter
    def threshold(self, value: float | None) -> None:
        """
        Set the threshold used for analysis.
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    def analyze(self, frame_data: list[float], width: int, height: int) -> DetectionResult:
        """
        Analyze the given frame data to detect its source resolution.

        This only works for grayscale images.

        Parameters
        ----------
        frame_data: :type:`list[float]`
            The pixel data of the image frame in a flat list, from 0.0 to 1.0.
        width: :type:`int`
            The width of the image frame.
        height: :type:`int`
            The height of the image frame.

        Returns
        -------
        :type:`DetectionResult`
            The result of the resolution detection.

        Raises
        ------
        ValueError
            If the threshold is set to an invalid value.
        RuntimeError
            If the analysis fails.
        """
        ...
