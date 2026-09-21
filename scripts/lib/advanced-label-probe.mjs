// Exercise engaged values: Auto-only screenshots do not expose reset-label collisions.
export async function advancedLabelProbe(page, report) {
  const results = [];
  for (const mode of ['Track Master', 'Album Master']) {
    await page.getByRole('button', { name: mode, exact: true }).click();
    for (const label of ['Width', 'Warmth', 'Presence/Air']) {
      const input = page.getByRole('spinbutton', { name: `${label} value`, exact: true });
      await input.fill('0');
      await input.press('Enter');
    }
    const fields = await page.locator('.rail-card-advanced .advanced-grid .adv-field').evaluateAll(nodes => nodes.map(field => {
      const label = field.querySelector('.adv-label');
      const range = document.createRange();
      range.selectNodeContents(label.firstChild);
      const rect = node => {
        const r = node.getBoundingClientRect();
        return { left: r.left, right: r.right, top: r.top, bottom: r.bottom };
      };
      const pieces = [rect(range), rect(field.querySelector('.adv-value'))];
      const reset = field.querySelector('.adv-auto-reset');
      if (reset) pieces.push(rect(reset));
      const bounds = rect(field);
      const overlaps = pieces.some((a, i) => pieces.slice(i + 1).some(b =>
        Math.min(a.right, b.right) - Math.max(a.left, b.left) > 1 &&
        Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top) > 1));
      const outside = pieces.some(p => p.left < bounds.left - 1 || p.right > bounds.right + 1);
      return { label: label.firstChild.textContent, overlaps, outside, pieces, bounds };
    }));
    for (const field of fields) {
      if (field.overlaps || field.outside) report(`${mode}: ${field.label} text overlaps or escapes its control: ${JSON.stringify(field)}`);
    }
    results.push({ mode, fields });
    for (const label of ['Width', 'Warmth', 'Presence/Air']) {
      await page.getByRole('button', { name: `Reset ${label} to Auto`, exact: true }).click();
    }
  }
  return results;
}
