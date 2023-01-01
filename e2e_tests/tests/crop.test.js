const sizeOf = require("image-size");

describe("Cropping Tests", () => {
  const sizes = [
    [123, 300],
    [402, 392],
    [1000, 600],
    [123, 560],
  ];
  test.each(sizes)(
    "Gets image at width %s and height %s",
    async (width, height) => {
      let res = await fetch(
        `http://127.0.0.1:3000/test_card_sml.png?width=${width}&height=${height}`
      );
      let imgBuffer = Buffer.from(await res.arrayBuffer());
      let size = sizeOf(imgBuffer);
      expect(size.width).toBe(width);
      expect(size.height).toBe(height);
      expect(imgBuffer).toMatchImageSnapshot();
    }
  );
});
