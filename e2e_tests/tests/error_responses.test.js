const { ROOT_URL } = require("./config");

describe("Error Response Tests", () => {
  test("404 for nonexistent image", async () => {
    const res = await fetch(
      ROOT_URL + "/this_image_definitely_does_not_exist.png"
    );
    expect(res.status).toBe(404);
  });

  test("404 for image in nonexistent directory", async () => {
    const res = await fetch(ROOT_URL + "/no/such/path/image.png");
    expect(res.status).toBe(404);
  });

  test("404 for path traversal attempt", async () => {
    const res = await fetch(ROOT_URL + "/../../etc/passwd");
    expect(res.status).toBe(404);
  });

  test("404 for URL-encoded path traversal", async () => {
    const res = await fetch(
      ROOT_URL + "/%2e%2e/%2e%2e/%2e%2e/etc/passwd"
    );
    expect(res.status).toBe(404);
  });

  test("400 for invalid width parameter", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?width=abc");
    expect(res.status).toBe(400);
  });

  test("400 for invalid height parameter", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?height=xyz");
    expect(res.status).toBe(400);
  });

  test("400 for invalid filter parameter", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?filter=invalid");
    expect(res.status).toBe(400);
  });

  test("400 for invalid oversize handling parameter", async () => {
    const res = await fetch(
      ROOT_URL + "/test_card_sml.png?oversizehandling=wrong"
    );
    expect(res.status).toBe(400);
  });

  test("400 for invalid colour parameter", async () => {
    const res = await fetch(ROOT_URL + "/test_card_sml.png?colour=wrong");
    expect(res.status).toBe(400);
  });
});
