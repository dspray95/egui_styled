"""Downsize the showcase's raw captures into the README's WebP screenshots (needs Pillow)."""

import sys
from pathlib import Path

from PIL import Image

SHOTS = ["dashboard_dark", "dashboard_brutal", "player", "playground"]
WIDTH = 1224  # 3x the ~400 px each tile gets in a GitHub README column.

src, dst = Path(sys.argv[1]), Path(sys.argv[2])
for name in SHOTS:
    im = Image.open(src / f"{name}.png").convert("RGB")
    im = im.resize((WIDTH, round(im.height * WIDTH / im.width)), Image.LANCZOS)
    im.save(dst / f"{name}.webp", "WEBP", quality=90, method=6)
    print(f"{name}.webp", (dst / f"{name}.webp").stat().st_size // 1024, "KB")
