from pathlib import Path

from PIL import Image

import resdet

CURRENT_DIR = Path(__file__).resolve().parent


def test_image_size_detection():
    img_file = Image.open(CURRENT_DIR / "img_input.png")
    assert img_file.width == 871
    assert img_file.height == 545
    # resize 150%
    img_file = img_file.resize((int(img_file.width * 1.5), int(img_file.height * 1.5)))

    width, height = img_file.size
    assert width == 1306
    assert height == 817

    analysis = resdet.Analysis()
    gray_data = resdet.normalize_image_gray(img_file.convert("L").getdata())
    result = analysis.analyze(gray_data, width, height)

    best_height = result.best_height()
    assert best_height is not None, "Best height should not be None"
    best_width = result.best_width()
    assert best_width is not None, "Best width should not be None"

    assert best_width.size == 871
    assert best_height.size == 545


def test_lib_version():
    version = resdet.lib_version()
    assert version == "2.1.1"
