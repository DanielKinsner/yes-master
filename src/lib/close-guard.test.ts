import { describe, it, expect } from "vitest";
import { shouldConfirmClose } from "./close-guard";

// S6.8: quitting while a render/export is in flight would discard work with no
// warning. The window close handler asks for confirmation only when something
// is actually running; an idle app closes immediately.
describe("shouldConfirmClose", () => {
  it("allows immediate close when nothing is in flight", () => {
    expect(
      shouldConfirmClose({
        isExporting: false,
        isRendering: false,
        albumRendering: false,
      }),
    ).toBe(false);
  });

  it("asks to confirm while an export is in flight", () => {
    expect(
      shouldConfirmClose({
        isExporting: true,
        isRendering: false,
        albumRendering: false,
      }),
    ).toBe(true);
  });

  it("asks to confirm while a track render is in flight", () => {
    expect(
      shouldConfirmClose({
        isExporting: false,
        isRendering: true,
        albumRendering: false,
      }),
    ).toBe(true);
  });

  it("asks to confirm while an album render is in flight", () => {
    expect(
      shouldConfirmClose({
        isExporting: false,
        isRendering: false,
        albumRendering: true,
      }),
    ).toBe(true);
  });
});
