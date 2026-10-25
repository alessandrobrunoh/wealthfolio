import { describe, expect, it } from "vitest";

import { PRESET_FLAGS } from "./rule-preset-constants";

describe("PRESET_FLAGS", () => {
  it("uses the Italian flag for the Italy preset", () => {
    expect(PRESET_FLAGS.it).toBe("🇮🇹");
  });
});
