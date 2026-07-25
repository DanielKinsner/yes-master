// Capture-endpoint security pin (hardening plan, Workstream F).
//
// The 2026-07-03 security pass found there is NO capture backend yet:
// SIGNUP_ENDPOINT ships empty and the form renders a disabled
// "opening soon" state. These tests pin the safety posture of both
// states so wiring a provider later can't silently regress them:
//   * unwired: controls disabled, and even a forced submit never
//     produces a network request;
//   * wired: exactly one POST to the configured endpoint, urlencoded,
//     carrying the email field and nothing else;
//   * failure: a visible error state, never an unhandled rejection.

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";

const mockConfig = vi.hoisted(() => ({ endpoint: "", field: "email" }));
vi.mock("./signup-config", () => ({
  get SIGNUP_ENDPOINT() {
    return mockConfig.endpoint;
  },
  get SIGNUP_FIELD() {
    return mockConfig.field;
  },
}));

import BetaSignup from "./BetaSignup";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;

async function renderSignup(): Promise<HTMLDivElement> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
  await act(async () => {
    root?.render(<BetaSignup />);
  });
  return container;
}

async function typeEmail(container: HTMLElement, value: string) {
  const input = container.querySelector("input[type=email]");
  expect(input).toBeInstanceOf(HTMLInputElement);
  await act(async () => {
    const setter = Object.getOwnPropertyDescriptor(
      HTMLInputElement.prototype,
      "value",
    )?.set;
    setter?.call(input, value);
    input?.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

async function submitForm(container: HTMLElement) {
  const form = container.querySelector("form");
  expect(form).toBeInstanceOf(HTMLFormElement);
  await act(async () => {
    form?.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  });
}

afterEach(async () => {
  await act(async () => {
    root?.unmount();
  });
  root = null;
  document.body.innerHTML = "";
  mockConfig.endpoint = "";
  mockConfig.field = "email";
  vi.unstubAllGlobals();
});

it("unwired endpoint: controls disabled and a forced submit sends nothing", async () => {
  const fetchSpy = vi.fn();
  vi.stubGlobal("fetch", fetchSpy);
  mockConfig.endpoint = "";

  const container = await renderSignup();
  const input = container.querySelector<HTMLInputElement>("input[type=email]");
  const button = container.querySelector<HTMLButtonElement>("button[type=submit]");
  expect(input?.disabled).toBe(true);
  expect(button?.disabled).toBe(true);
  // U6: the label used to read "Email updates opening soon". "Soon" is a
  // roadmap word the landing brief bans, and no provider is selected, so
  // "opening" promised a schedule for something undecided.
  expect(button?.textContent).toBe("Email updates");
  expect(container.textContent).not.toMatch(/soon/i);

  // The inert control must not look like the live CTA, and must carry a
  // visible, associated reason rather than a bare greyed-out button.
  expect(button?.className).not.toContain("bg-gradient-to-b");
  const reasonId = button?.getAttribute("aria-describedby");
  expect(reasonId).toBeTruthy();
  const reason = container.querySelector(`#${reasonId}`);
  expect(reason?.textContent).toContain("Email updates are not open");

  // Even if the disabled attribute is stripped (devtools, extensions),
  // the submit handler itself must refuse to build a request.
  await submitForm(container);
  expect(fetchSpy).not.toHaveBeenCalled();
});

it("wired endpoint: posts exactly the email field, urlencoded, then confirms", async () => {
  const fetchSpy = vi.fn().mockResolvedValue({ ok: true });
  vi.stubGlobal("fetch", fetchSpy);
  mockConfig.endpoint = "https://example.test/subscribe";

  const container = await renderSignup();
  await typeEmail(container, "dan+beta@example.com");
  await submitForm(container);

  expect(fetchSpy).toHaveBeenCalledTimes(1);
  const [url, init] = fetchSpy.mock.calls[0] as [string, RequestInit];
  expect(url).toBe("https://example.test/subscribe");
  expect(init.method).toBe("POST");
  expect(init.headers).toEqual({
    "Content-Type": "application/x-www-form-urlencoded",
  });
  // The body carries the email and nothing else — no tracking fields,
  // no metadata, exactly what the copy promises.
  expect(init.body).toBe(`email=${encodeURIComponent("dan+beta@example.com")}`);
  // U6: this used to promise to "save your founder price" — the C-22
  // entitlement whose terms are undecided (owner queue row 1). It was
  // unreachable while the form is unwired, and would have shipped the moment a
  // provider was wired. A test asserting an owner-blocked claim is worse than
  // no test: it makes the claim look approved.
  expect(container.textContent).toContain("You're on the list");
  expect(container.textContent).not.toMatch(/founder price|keep \$29/i);
});

it("provider failure surfaces the error state instead of throwing", async () => {
  const fetchSpy = vi.fn().mockRejectedValue(new Error("network down"));
  vi.stubGlobal("fetch", fetchSpy);
  mockConfig.endpoint = "https://example.test/subscribe";

  const container = await renderSignup();
  await typeEmail(container, "dan@example.com");
  await submitForm(container);

  expect(container.textContent).toContain("Something went wrong");
  expect(container.textContent).toContain("hello@yesmaster.app");
});

it("non-OK provider response also lands in the error state", async () => {
  const fetchSpy = vi.fn().mockResolvedValue({ ok: false });
  vi.stubGlobal("fetch", fetchSpy);
  mockConfig.endpoint = "https://example.test/subscribe";

  const container = await renderSignup();
  await typeEmail(container, "dan@example.com");
  await submitForm(container);

  expect(container.textContent).toContain("Something went wrong");
});
