"""Run with Python 3 and the path to ofs_sdk.dll or libofs_sdk.so."""
import ctypes
import math
from pathlib import Path
import sys


class Estimate(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("total_eur", ctypes.c_double),
        ("labor_hours", ctypes.c_double),
        ("glass_eur", ctypes.c_double),
    ]


if __name__ == "__main__":
    library = ctypes.CDLL(str(Path(sys.argv[1]).resolve()))
    estimate = library.ofs_estimate_fixed_v1
    estimate.argtypes = [ctypes.c_double, ctypes.c_double]
    estimate.restype = Estimate
    result = estimate(1000, 1200)
    assert result.status == 0 and result.total_eur > result.glass_eur > 0
    assert result.labor_hours > 0
    for width, height in [(100, 1200), (1000, 5000), (math.nan, 1200)]:
        invalid = estimate(width, height)
        assert invalid.status == 1 and invalid.total_eur == 0
    print(f"1000 × 1200 mm: EUR {result.total_eur:.2f}, {result.labor_hours:.2f} h")
