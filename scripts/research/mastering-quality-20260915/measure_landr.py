"""Remeasure archived same-source masters without any audio processing."""
import json
from analyze_music import OUT,metric
files=sorted((OUT.parents[1]/'tests for presets').glob('LANDR*.wav'))
assert len(files)==3, files
rows=[metric(p,'landr_'+p.stem) for p in files]
(OUT/'landr-current-measurements.json').write_text(json.dumps(rows,indent=2))
print('Measured saved LANDR files',len(rows))
