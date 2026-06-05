export function SettingsGroup({
  title,
  rows,
}: {
  title: string;
  rows: Array<[string, string]>;
}) {
  return (
    <section className="settings-group">
      <h3>{title}</h3>
      <dl>
        {rows.map(([label, value]) => (
          <div className="settings-row" key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
