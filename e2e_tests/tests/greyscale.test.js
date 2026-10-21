const { ROOT_URL } = require("./config");

describe("Greyscale Tests", () => {
  test("greyscale flag returns 200", async () => {
    const res = await fetch(
      ROOT_URL + "/test_card_sml.png?colour=greyscale"
    );
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    expect(imgBuffer.length).toBeGreaterThan(0);
  });

  test("greyscale with resize works", async () => {
    const res = await fetch(
      ROOT_URL + "/test_card_sml.png?width=50&colour=greyscale"
    );
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    expect(imgBuffer.length).toBeGreaterThan(0);
  });

  test("greyscale with both dimensions", async () => {
    const res = await fetch(
      ROOT_URL + "/test_card_sml.png?width=100&height=100&colour=greyscale"
    );
    expect(res.status).toBe(200);
    const imgBuffer = Buffer.from(await res.arrayBuffer());
    expect(imgBuffer.length).toBeGreaterThan(0);
  });
});
