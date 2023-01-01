const sizeOf = require("image-size");

describe("Resizing Tests", () => {
  const sizes = [123, 300, 402, 392];
  test.each(sizes)("Gets image at width %s", async (width) => {
    let res = await fetch(
      "http://127.0.0.1:3000/test_card_sml.png?width=" + width
    );
    let imgBuffer = Buffer.from(await res.arrayBuffer());
    let size = sizeOf(imgBuffer);
    expect(size.width).toBe(width);
    expect(imgBuffer).toMatchImageSnapshot();
  });
  test.each(sizes)("Gets image at height %s", async (height) => {
    let res = await fetch(
      "http://127.0.0.1:3000/test_card_sml.png?height=" + height
    );
    let imgBuffer = Buffer.from(await res.arrayBuffer());
    let size = sizeOf(imgBuffer);
    expect(size.height).toBe(height);
    expect(imgBuffer).toMatchImageSnapshot();
  });
});
