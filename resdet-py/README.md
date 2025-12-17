# resdet

Safe Python bindings to libresdet for detecting upscaled images

[![PyPI version](https://img.shields.io/pypi/v/resdet.svg)](https://pypi.org/project/resdet/)

## Usage

`resdet` can be installed from PyPI:

```bash
pip install resdet
```

And can be used as follows:

```python
import resdet
from PIL import Image

img = Image.open("image_to_test.png")
img = img.convert("L") # resdet only supports grayscale images
width, height = img.size

normalized_data = resdet.normalize_image_data(list(img.getdata())) # Normalize for resdet

analysis = resdet.Analysis()
result = analysis.analyze(normalized_data, width, height)

best_width = result.best_width()
assert best_width is not None, "No width detected"
best_height = result.best_height()
assert best_height is not None, "No height detected"

print(f"Input image size: {width}x{height}")
print(f"Best matching resdet size: {best_width}x{best_height}")
```

## Thread Safety

The `resdet` library and the bindings should be thread-safe, allowing multiple analyses to be run in parallel without issues.

You can freely share `Analysis` instances between threads, as long as each thread uses its own `analyze` calls.

## License

This project is dual-licensed under the MIT License and the LGPL-2.1 License
- the source code of this project is licensed under the MIT License
- the distributed builds/wheels are licensed under the LGPL-2.1 License, the same as the underlying `resdet` library
