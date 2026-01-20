from playwright.sync_api import sync_playwright

def verify_logo():
    with sync_playwright() as p:
        browser = p.chromium.launch()
        page = browser.new_page()
        page.goto("http://localhost:8080")

        # Wait for the logo to be visible
        page.wait_for_selector(".logo-eagle")

        # Take a screenshot
        page.screenshot(path="opensentinel_logo_integrated.png")

        # Verify the logo src
        logo_src = page.get_attribute(".logo-eagle", "src")
        print(f"Logo Source: {logo_src}")

        if logo_src == "azanian_eagle_logo.png":
            print("Logo integrated successfully.")
        else:
            print(f"Logo integration failed. Expected 'azanian_eagle_logo.png', got '{logo_src}'")

        browser.close()

if __name__ == "__main__":
    verify_logo()
