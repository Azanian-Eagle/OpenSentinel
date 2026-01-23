import asyncio
import os
import subprocess
import time
from playwright.async_api import async_playwright

async def run():
    # Start the server
    server_process = subprocess.Popen(
        ["python3", "-m", "http.server", "3000", "--directory", "docs"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )

    # Wait for server to start
    time.sleep(2)

    try:
        async with async_playwright() as p:
            browser = await p.chromium.launch()

            # Define viewports
            viewports = {
                "desktop": {"width": 1920, "height": 1080},
                "mobile": {"width": 375, "height": 812, "is_mobile": True, "has_touch": True}
            }

            os.makedirs("verification_images", exist_ok=True)

            for page_name in ["index.html", "demo.html"]:
                for vp_name, vp_config in viewports.items():
                    context = await browser.new_context(
                        viewport={"width": vp_config["width"], "height": vp_config["height"]},
                        user_agent="Mozilla/5.0 (iPhone; CPU iPhone OS 11_0 like Mac OS X) AppleWebKit/604.1.38 (KHTML, like Gecko) Version/11.0 Mobile/15A372 Safari/604.1" if vp_name == "mobile" else None
                    )
                    page = await context.new_page()

                    url = f"http://localhost:3000/{page_name}"
                    print(f"Navigating to {url} on {vp_name}...")
                    await page.goto(url)

                    # Wait for content to load (animations, etc.)
                    await page.wait_for_timeout(2000)

                    screenshot_path = f"verification_images/{page_name.split('.')[0]}_{vp_name}.png"
                    await page.screenshot(path=screenshot_path, full_page=True)
                    print(f"Saved {screenshot_path}")

                    await context.close()

            await browser.close()
    finally:
        server_process.terminate()

if __name__ == "__main__":
    asyncio.run(run())
