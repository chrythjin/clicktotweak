import { describe, expect, it } from "vitest";
import {
  ALPHA_MAX,
  ALPHA_MIN,
  alphaToPercentLabel,
  percentToAlpha,
} from "../lib/transparency";

describe("alphaToPercentLabel", () => {
  it("maps the slider bounds to 10% and 100%", () => {
    expect(alphaToPercentLabel(ALPHA_MIN)).toBe("10");
    expect(alphaToPercentLabel(ALPHA_MAX)).toBe("100");
  });

  it("renders a whole percentage", () => {
    expect(alphaToPercentLabel(128)).toBe("50");
  });

  it("clamps values outside the supported range", () => {
    expect(alphaToPercentLabel(0)).toBe("10");
    expect(alphaToPercentLabel(999)).toBe("100");
  });
});

describe("percentToAlpha", () => {
  it("round trips with the label", () => {
    expect(percentToAlpha(100)).toBe(255);
    expect(percentToAlpha(50)).toBe(128);
    expect(percentToAlpha(0)).toBe(0);
  });

  it("clamps the percentage to the valid range", () => {
    expect(percentToAlpha(-20)).toBe(0);
    expect(percentToAlpha(150)).toBe(255);
  });
});
