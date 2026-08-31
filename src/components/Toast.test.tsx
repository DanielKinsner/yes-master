import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { Toast } from "./Toast";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

async function render(node: ReactNode): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(node);
  });
  return { container, root };
}

async function click(el: HTMLElement): Promise<void> {
  await act(async () => {
    el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("Toast", () => {
  it("renders a plain toast with no action by default", async () => {
    const { container, root } = await render(
      <Toast message="Project saved." tone="ok" onClose={() => {}} />,
    );
    expect(container.querySelector(".toast")?.textContent).toContain("Project saved.");
    expect(container.querySelector(".toast-action")).toBeNull();
    await act(async () => {
      root.unmount();
    });
  });

  it("renders an enabled action and fires it on click (Slice 7b)", async () => {
    const onClick = vi.fn();
    const { container, root } = await render(
      <Toast
        message="Update available — v1.2.3"
        tone="info"
        onClose={() => {}}
        actions={[{ label: "Restart to update", onClick }]}
      />,
    );
    const btn = container.querySelector<HTMLButtonElement>(".toast-action");
    expect(btn?.textContent).toBe("Restart to update");
    expect(btn?.disabled).toBe(false);
    expect(btn?.getAttribute("title")).toBeNull();

    await click(btn!);
    expect(onClick).toHaveBeenCalledTimes(1);
    await act(async () => {
      root.unmount();
    });
  });

  it("disables the action with an explanatory title while work is in progress", async () => {
    const onClick = vi.fn();
    const { container, root } = await render(
      <Toast
        message="Update available — v1.2.3"
        tone="info"
        onClose={() => {}}
        actions={[
          {
            label: "Restart to update",
            onClick,
            disabled: true,
            disabledTitle:
              "Finishing your export first — this re-enables when it's done.",
          },
        ]}
      />,
    );
    const btn = container.querySelector<HTMLButtonElement>(".toast-action");
    expect(btn?.disabled).toBe(true);
    expect(btn?.getAttribute("title")).toBe(
      "Finishing your export first — this re-enables when it's done.",
    );
    await act(async () => {
      root.unmount();
    });
  });

  it("renders multiple actions in order and a disabled one never fires (audit L-03)", async () => {
    const onRetry = vi.fn();
    const onManual = vi.fn();
    const { container, root } = await render(
      <Toast
        message="Update couldn't install. Retry, or download it manually."
        tone="warn"
        onClose={() => {}}
        actions={[
          {
            label: "Retry",
            onClick: onRetry,
            disabled: true,
            disabledTitle:
              "Finishing your export first — this re-enables when it's done.",
          },
          { label: "Download manually", onClick: onManual },
        ]}
      />,
    );
    const buttons = Array.from(
      container.querySelectorAll<HTMLButtonElement>(".toast-actions .toast-action"),
    );
    expect(buttons.map((b) => b.textContent)).toEqual([
      "Retry",
      "Download manually",
    ]);

    await click(buttons[0]);
    expect(onRetry).not.toHaveBeenCalled();
    await click(buttons[1]);
    expect(onManual).toHaveBeenCalledTimes(1);

    // The dismiss button survives alongside stacked actions.
    expect(container.querySelector(".toast-close")).not.toBeNull();
    await act(async () => {
      root.unmount();
    });
  });
});
