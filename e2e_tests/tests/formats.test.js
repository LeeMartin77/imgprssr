const sizeOf = require("image-size");
const { ROOT_URL } = require("./config");

describe("Image Format Tests", () => {
  test("PNG input produces PNG output", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?width=100");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.width).toBe(100);
    expect(size.height).toBeGreaterThan(0);
  });

  test("WebP input produces WebP output", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.webp?width=100");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.width).toBe(100);
    expect(size.height).toBeGreaterThan(0);
  });

  test("WebP with greyscale", async () => {
    const res = await fetch(
      ROOT_URL + "/test_card_sml.webp?width=80&colour=greyscale"
    );
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.width).toBe(80);
  });
});
