const sizeOf = require("image-size");
const { ROOT_URL } = require("./config");

describe("Edge Case Tests", () => {
  test("small width (1px)", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?width=1");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.width).toBe(1);
  });

  test("small height (1px)", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?height=1");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.height).toBe(1);
  });

  test("large width", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?width=4000");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    // Clamp mode prevents upscaling — 1280 < 4000, so image stays at original width
    expect(size.width).toBe(1280);
  });

  test("large height", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?height=4000");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    // Clamp mode prevents upscaling — 720 < 4000, so image stays at original height
    expect(size.height).toBe(720);
  });

  test("both dimensions specified", async () => {
    const res = await fetch(
      ROOT_URL + "/test_card_sml.png?width=200&height=150"
    );
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.width).toBe(200);
    expect(size.height).toBe(150);
  });

  test("all parameters combined", async () => {
    const res = await fetch(
      ROOT_URL +
        "/test_card_sml.png?width=120&height=100&filter=gaussian&colour=greyscale&oversizehandling=clamp"
    );
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.width).toBe(120);
    expect(size.height).toBe(100);
  });

  test("clamp prevents upscaling width", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?width=5");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    // Clamp mode should not upscale below original size
    expect(size.width).toBeLessThanOrEqual(100);
  });

  test("clamp prevents upscaling height", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?height=5");
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    const size = sizeOf(imgBuffer);
    expect(size.height).toBeLessThanOrEqual(100);
  });
});
