import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, expect, it } from "vitest";

import BetaDownload from "./BetaDownload";
import { BETA_DOWNLOAD_URL } from "./release-config";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;

afterEach(async () => {
  await act(async () => {
    root?.unmount();
  });
  root = null;
  document.body.innerHTML = "";
});

it("offers an ungated GitHub download for the Windows and universal Mac beta", async () => {
  const container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);

  await act(async () => {
    root?.render(<BetaDownload />);
  });

  const link = container.querySelector<HTMLAnchorElement>("a");
  expect(link?.textContent).toContain("Download the free beta");
  expect(link?.href).toBe(BETA_DOWNLOAD_URL);
  expect(link?.target).toBe("_blank");
  expect(link?.rel).toContain("noreferrer");
  expect(container.textContent).toContain("No email required");
  expect(container.textContent).toContain("Windows");
  expect(container.textContent).toContain("universal Mac");
});
