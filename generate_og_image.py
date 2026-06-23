from PIL import Image, ImageDraw
import numpy as np

width, height = 1200, 630

# Create a dark background
arr = np.zeros((height, width, 3), dtype=np.uint8)
arr[:, :, 0] = 5
arr[:, :, 1] = 5
arr[:, :, 2] = 5

# Add a subtle gradient accent line at the top and bottom
for x in range(width):
    t = x / width
    r = int((1 - t) * 0 + t * 189)
    g = int((1 - t) * 243 + t * 0)
    b = int((1 - t) * 255 + t * 255)

    # Bottom 10 pixels
    for y in range(height - 10, height):
        arr[y, x] = [r, g, b]

    # Top 10 pixels
    for y in range(0, 10):
        arr[y, x] = [r, g, b]

img = Image.fromarray(arr)

# Open logo
logo = Image.open('docs/assets/images/logo.png').convert("RGBA")

# Resize logo to fit nicely within 630px height, maybe 450x450
logo.thumbnail((450, 450), Image.Resampling.LANCZOS)

logo_w, logo_h = logo.size

# Calculate position to center the logo
offset = ((width - logo_w) // 2, (height - logo_h) // 2)

# Paste logo onto background, using the logo's alpha channel as a mask
img.paste(logo, offset, logo)

# Save the resulting image
img.save('docs/assets/images/og-image.png')
