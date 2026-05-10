from PIL import Image, ImageDraw

def generate_icon():
    size = 256
    bg_color = (30, 30, 46)      # #1E1E2E
    accent_color = (137, 180, 250) # #89B4FA
    
    # Create a square image with transparent background
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    center = size // 2
    r = int(size * 0.4)
    
    # Draw circle (muted accent)
    muted_accent = (accent_color[0], accent_color[1], accent_color[2], 100) # Slightly more transparent
    draw.ellipse([center - r, center - r, center + r, center + r], outline=muted_accent, width=10)
    
    # Draw pulse line
    p_len = int(r * 1.8)
    y = center
    x_start = center - p_len // 2
    x_end = center + p_len // 2
    
    # Points for the pulse
    points = [
        (x_start, y),
        (center - int(r * 0.25), y),
        (center, y - int(r * 0.55)),
        (center + int(r * 0.25), y),
        (x_end, y)
    ]
    
    draw.line(points, fill=accent_color + (255,), width=14, joint='curve')
    
    # Draw dots at ends
    dot_r = 12
    draw.ellipse([x_start - dot_r, y - dot_r, x_start + dot_r, y + dot_r], fill=accent_color + (255,))
    draw.ellipse([x_end - dot_r, y - dot_r, x_end + dot_r, y + dot_r], fill=accent_color + (255,))
    
    # Save as PNG first (for main.rs)
    img.save('icon.png', format='PNG')
    
    # Save as ICO (for windows window icon)
    # Note: .ico supports transparency in RGBA mode
    img.save('icon.ico', format='ICO', sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print("icon.png and icon.ico generated successfully with transparency.")

if __name__ == "__main__":
    generate_icon()
