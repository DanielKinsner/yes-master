// U4 — the beta tester's feedback contract, asserted rather than eyeballed.
//
// These forms are the only structured route a tester has into the project, and
// they are easy to break silently: GitHub renders a malformed issue form as a
// plain textarea with no error anywhere, so a broken form looks like a working
// one until reports start arriving with none of the fields anyone needs.
//
// So the forms are parsed as REAL YAML (not regex-matched) — a hand-rolled
// check would happily pass a file GitHub rejects, which is precisely the
// failure worth catching.

import { readFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { parse } from "yaml";

const root = process.cwd();

function read(relative: string): string {
  return readFileSync(resolve(root, relative), "utf8");
}

function parseYaml(relative: string): Record<string, unknown> {
  return parse(read(relative)) as Record<string, unknown>;
}

interface FormField {
  type: string;
  id?: string;
  attributes?: { label?: string; value?: string; description?: string };
  validations?: { required?: boolean };
}

interface IssueForm {
  name: string;
  description: string;
  body: FormField[];
  labels?: string[];
  title?: string;
}

const BUG_PATH = ".github/ISSUE_TEMPLATE/beta-bug.yml";
const FEEDBACK_PATH = ".github/ISSUE_TEMPLATE/beta-feedback.yml";
const CONFIG_PATH = ".github/ISSUE_TEMPLATE/config.yml";
const GUIDE_PATH = "docs/BETA_TESTING.md";
const INSTALL_PATH = "docs/BETA_INSTALL.md";

const bug = parseYaml(BUG_PATH) as unknown as IssueForm;
const feedback = parseYaml(FEEDBACK_PATH) as unknown as IssueForm;

/** Fields the tester must fill in before the form can be submitted. */
function requiredIds(form: IssueForm): string[] {
  return form.body
    .filter((field) => field.validations?.required)
    .map((field) => field.id ?? "")
    .filter(Boolean);
}

describe("U4 — issue forms are valid GitHub issue forms", () => {
  it("both parse as YAML and carry the keys GitHub requires", () => {
    for (const [path, form] of [
      [BUG_PATH, bug],
      [FEEDBACK_PATH, feedback],
    ] as const) {
      expect(form, `${path} parsed to nothing`).toBeTruthy();
      expect(typeof form.name, `${path} name`).toBe("string");
      expect(typeof form.description, `${path} description`).toBe("string");
      expect(Array.isArray(form.body), `${path} body`).toBe(true);
      expect(form.body.length).toBeGreaterThan(0);
    }
  });

  it("uses only field types GitHub understands, and ids where it needs them", () => {
    const ALLOWED = new Set(["markdown", "input", "textarea", "dropdown", "checkboxes"]);
    for (const [path, form] of [
      [BUG_PATH, bug],
      [FEEDBACK_PATH, feedback],
    ] as const) {
      for (const field of form.body) {
        expect(ALLOWED.has(field.type), `${path}: bad type "${field.type}"`).toBe(true);
        // Every field except markdown must be addressable, or its answer
        // arrives in the issue body with nothing naming it.
        if (field.type !== "markdown") {
          expect(field.id, `${path}: ${field.attributes?.label} has no id`).toBeTruthy();
          expect(field.attributes?.label, `${path}: field ${field.id} has no label`)
            .toBeTruthy();
        }
      }
    }
  });

  it("disables blank issues so a report cannot arrive unstructured", () => {
    const config = parseYaml(CONFIG_PATH) as { blank_issues_enabled?: boolean };
    expect(config.blank_issues_enabled).toBe(false);
  });
});

describe("U4 — the bug form demands what reproduction actually needs", () => {
  const required = requiredIds(bug);

  it("requires version, OS, install type, steps, expected, and actual", () => {
    // Version and OS are the two nobody volunteers and nothing can be
    // reproduced without.
    for (const id of [
      "version",
      "os",
      "os-version",
      "install-type",
      "steps",
      "expected",
      "actual",
    ]) {
      expect(required, `bug form is missing required field "${id}"`).toContain(id);
    }
  });

  it("requires the audio metadata that changes engine behaviour", () => {
    // Already-mastered sources drive a different adaptive path, so a report
    // without it is ambiguous in a way that wastes a round trip.
    expect(required).toContain("source-audio");
    expect(required).toContain("already-mastered");
  });

  it("asks whether diagnostics exist, and allows 'the app was not responding'", () => {
    expect(required).toContain("diagnostics");
    const diagnostics = bug.body.find((f) => f.id === "diagnostics") as
      | (FormField & { attributes: { options: string[] } })
      | undefined;
    // A form that only offers "attached / not attached" quietly punishes the
    // exact case worth hearing about — the crash that stopped you saving one.
    expect(
      diagnostics?.attributes.options.some((o) => /not responding/i.test(o)),
    ).toBe(true);
  });
});

describe("U4 — the feedback form treats sound as a first-class report", () => {
  const required = requiredIds(feedback);

  it("requires area, description, and severity", () => {
    for (const id of ["area", "what", "severity"]) {
      expect(required, `feedback form is missing required field "${id}"`).toContain(id);
    }
  });

  it("covers preset, loudness, Intensity, and confusion as reportable areas", () => {
    const area = feedback.body.find((f) => f.id === "area") as
      | (FormField & { attributes: { options: string[] } })
      | undefined;
    const options = (area?.attributes.options ?? []).join(" | ").toLowerCase();
    for (const topic of ["preset", "loudness", "intensity", "confus"]) {
      expect(options, `feedback areas do not cover "${topic}"`).toContain(topic);
    }
  });

  it("asks permission before following up, and does not collect an address", () => {
    expect(required).toContain("follow-up");
    const followUp = feedback.body.find((f) => f.id === "follow-up");
    // Follow-up happens in the public thread. Anything else would imply an
    // email capture this project does not have.
    expect(followUp?.attributes?.description ?? "").toMatch(/do not collect email/i);
  });
});

describe("U4 — privacy warnings come BEFORE anything that could carry audio", () => {
  it("puts the private-audio and public-issue warning in the first block", () => {
    for (const [path, form] of [
      [BUG_PATH, bug],
      [FEEDBACK_PATH, feedback],
    ] as const) {
      const first = form.body[0];
      expect(first.type, `${path}: first block is not markdown`).toBe("markdown");
      // Collapse whitespace before matching: the source is hard-wrapped, so
      // "a free GitHub\naccount" is the same sentence as "a free GitHub
      // account" and a raw substring check would report a missing disclosure
      // that is plainly there.
      const intro = (first.attributes?.value ?? "")
        .toLowerCase()
        .replace(/\s+/g, " ");
      // A warning printed under the upload box is read after the upload.
      expect(intro, `${path}: no private-audio warning up front`).toContain(
        "private audio",
      );
      expect(intro, `${path}: does not say the issue is public`).toContain("public");
      expect(intro, `${path}: does not disclose the GitHub account requirement`)
        .toContain("github account");
      // Must not imply a login that does not exist.
      expect(intro, `${path}: does not deny a YES Master account`).toContain(
        "no yes master account",
      );
    }
  });

  it("never asks for an email address or analytics consent", () => {
    for (const [path, form] of [
      [BUG_PATH, bug],
      [FEEDBACK_PATH, feedback],
    ] as const) {
      const labels = form.body
        .map((f) => `${f.attributes?.label ?? ""} ${f.attributes?.description ?? ""}`)
        .join(" ")
        .toLowerCase();
      expect(labels, `${path} asks for an email address`).not.toMatch(
        /your email|email address\?|enter your email/,
      );
      expect(labels, `${path} asks for analytics consent`).not.toMatch(
        /analytics|telemetry consent|usage data/,
      );
    }
  });

  it("offers no unsupported platform anywhere in the OS choices", () => {
    // Mobile is parked and Linux is deferred. Listing either invites reports
    // against a product that does not exist.
    const os = bug.body.find((f) => f.id === "os") as
      | (FormField & { attributes: { options: string[] } })
      | undefined;
    const options = (os?.attributes.options ?? []).join(" ").toLowerCase();
    expect(options).not.toMatch(/linux|ios|iphone|android/);
    expect(options).toMatch(/windows/);
    expect(options).toMatch(/macos/);
  });
});

describe("U4 — the beta guide is discoverable and honest", () => {
  const guide = read(GUIDE_PATH);

  it("exists and links the install guide", () => {
    expect(existsSync(resolve(root, GUIDE_PATH))).toBe(true);
    expect(guide).toContain("BETA_INSTALL.md");
    expect(existsSync(resolve(root, INSTALL_PATH))).toBe(true);
  });

  it("states the beta lifecycle without a deactivation threat", () => {
    expect(guide).toMatch(/installed build keeps working/i);
    expect(guide).toMatch(/no kill switch/i);
    expect(guide.toLowerCase()).toContain("optional");
    // Coercion is the failure the unit names, and it is asserted POSITIVELY —
    // the guide has to say feedback is not owed. Searching for a coercive
    // phrase instead does not work: the first version of this test matched the
    // guide's own promise NOT to do it ("no 'give feedback or lose access'")
    // and failed the document for containing its own reassurance.
    expect(guide).toMatch(/not obliged to report anything/i);
    expect(guide).toMatch(/will not be pressured for feedback/i);
  });

  it("says plainly that nothing is transmitted", () => {
    expect(guide).toMatch(/no telemetry/i);
    expect(guide).toMatch(/diagnostics report/i);
  });

  it("keeps the newsletter beside the download and marks it not-yet-open", () => {
    // The provider is an open owner question, so the conservative default has
    // to be visible in the guide as well as in the code.
    expect(guide).toMatch(/never in front of it|beside the download/i);
    expect(guide).toMatch(/not yet open|deliberately inactive/i);
    expect(guide).toMatch(/never gated/i);
  });

  it("does not promise a mobile download while mobile is parked", () => {
    expect(guide).toMatch(/mobile is not part of this beta/i);
  });

  it("names no concrete beta end date while that is unanswered", () => {
    // OWNER_INPUT_QUEUE: the beta end date is not decided. Public copy stays
    // time-boxed-but-undated until it is.
    expect(guide).toMatch(/time-boxed|limited period/i);
    expect(guide).not.toMatch(/\b20\d\d-\d\d-\d\d\b/);
  });
});

describe("U4 — the signup form stays safe-disabled until a provider is chosen", () => {
  it("ships no endpoint, so no address can be collected", () => {
    const config = read("src/landing/signup-config.ts");
    expect(config).toMatch(/SIGNUP_ENDPOINT\s*=\s*""/);
  });
});

describe("U4 — the release notes route a tester to both guides", () => {
  const workflow = read(".github/workflows/release.yml");

  it("links the install guide and the testing guide", () => {
    // A release page is where most testers arrive first. If the notes only
    // link install steps, the "what should I actually test, and what is
    // already known broken" page is never found.
    expect(workflow).toContain("docs/BETA_INSTALL.md");
    expect(workflow).toContain("docs/BETA_TESTING.md");
  });

  it("links both issue forms by their real template filenames", () => {
    // Template query params that do not match a real file silently drop the
    // tester onto the blank chooser — which blank_issues_enabled: false then
    // makes a dead end.
    for (const template of ["beta-bug.yml", "beta-feedback.yml"]) {
      expect(workflow, `release notes do not link ${template}`).toContain(
        `template=${template}`,
      );
      expect(
        existsSync(resolve(root, ".github/ISSUE_TEMPLATE", template)),
        `${template} is linked but does not exist`,
      ).toBe(true);
    }
  });

  it("repeats the public-issue and no-audio warning at the point of handoff", () => {
    expect(workflow).toMatch(/public and need a free GitHub account/i);
    expect(workflow).toMatch(/describe your audio rather than uploading/i);
  });
});

describe("U4 — local link check", () => {
  it("every relative doc link in the beta docs resolves to a real file", () => {
    for (const path of [GUIDE_PATH, INSTALL_PATH]) {
      const text = read(path);
      const links = [...text.matchAll(/\]\(([^)]+)\)/g)].map((m) => m[1]);
      for (const link of links) {
        if (/^(https?:|mailto:|#)/.test(link)) continue;
        const target = resolve(root, "docs", link.split("#")[0]);
        expect(existsSync(target), `${path}: broken link -> ${link}`).toBe(true);
      }
    }
  });
});
