// Exercise engaged values: Auto-only screenshots do not expose reset-label collisions.
export async function advancedLabelProbe(page, report) {
  const results = [];
  for (const mode of ['Track Master', 'Album Master']) {
    await page.getByRole('button', { name: mode, exact: true }).click();
    let automatic;
    for (const state of ['auto', 'mixed', 'manual']) {
      for (const [label, value] of [['Width', '1.2'], ['Warmth', '0.35'], ['Presence/Air', '0.4']]) {
        if (state === 'mixed' && label !== 'Width') continue;
        const input = page.getByRole('spinbutton', { name: `${label} value`, exact: true });
        await input.fill(state === 'auto' ? '' : value);
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
        const grid = rect(field.closest('.advanced-grid'));
        const labelBounds = rect(label.firstChild);
        const valueBounds = rect(field.querySelector('.adv-value'));
        const sliderBounds = rect(field.querySelector('input[type="range"]'));
        const positions = {
          label: labelBounds.top - grid.top,
          value: valueBounds.top - grid.top,
          slider: sliderBounds.top - grid.top,
          sliderInset: sliderBounds.left - bounds.left,
          sliderWidth: sliderBounds.right - sliderBounds.left,
          height: bounds.bottom - bounds.top,
        };
        const overlaps = pieces.some((a, i) => pieces.slice(i + 1).some(b =>
          Math.min(a.right, b.right) - Math.max(a.left, b.left) > 1 &&
          Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top) > 1));
        const outside = pieces.some(p => p.left < bounds.left - 1 || p.right > bounds.right + 1);
        return { label: label.firstChild.textContent, overlaps, outside, pieces, bounds,
          positions, headerGap: pieces[1].left - pieces[0].right };
      }));
      for (const field of fields) {
        if (field.overlaps || field.outside || field.headerGap < 4) report(`${mode} ${state}: ${field.label} text overlaps, crowds or escapes its control: ${JSON.stringify(field)}`);
      }
      for (const [left, right] of [[0, 1], [2, 3]]) {
        for (const key of ['label', 'value', 'slider', 'sliderInset', 'sliderWidth', 'height']) {
          if (Math.abs(fields[left].positions[key] - fields[right].positions[key]) > 1) report(`${mode} ${state}: ${key} is not aligned across the control row`);
        }
      }
      if (state === 'auto') automatic = fields;
      else fields.forEach((field, index) => {
        for (const key of ['label', 'value', 'slider', 'sliderInset', 'sliderWidth', 'height']) {
          if (Math.abs(field.positions[key] - automatic[index].positions[key]) > 1) report(`${mode} ${state}: ${field.label} ${key} moved after editing`);
        }
      });
      results.push({ mode, state, fields });
    }
    for (const label of ['Width', 'Warmth', 'Presence/Air']) {
      const reset = page.getByRole('button', { name: `Reset ${label} to Auto`, exact: true });
      await reset.click();
      if (await reset.isEnabled()) report(`${mode}: ${label} reset did not return to Auto`);
    }
  }
  return results;
}
