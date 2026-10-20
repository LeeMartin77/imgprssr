const { ROOT_URL } = require("./config");

describe("E2E Infrastructure Smoke Test", () => {
  test("server is reachable", async () => {
    const res = await fetch(ROOT_URL + "/nonexistent.png");
    // We expect a 404 (not found) — not a connection error.
    // If the server is unreachable, fetch() will throw.
    expect(res.status).toBe(404);
  });
});
