"""Emit the case lists defined by PROTOCOL.md §2/§5."""
import json
import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CASES = os.path.join(ROOT, "audit-output", "cases")
os.makedirs(CASES, exist_ok=True)

INPUTS = {f"input{i:02d}": f"fixtures/inputs/input{i:02d}.wav" for i in range(1, 9)}
SYN = lambda n: f"audit-output/synthetic/{n}"
VAR = lambda n: f"audit-output/variants/{n}"
PRESETS = ["universal", "clarity", "tape", "spatial", "oomph", "warmth", "punch", "loud", "custom"]

DEFAULT = {"preset": "universal", "intensity": 0.5, "delivery_profile": "streaming-universal"}


def dump(name, cases):
    with open(os.path.join(CASES, name), "w") as f:
        json.dump(cases, f, indent=1)
    print(name, len(cases))


# ---- controls -------------------------------------------------------------
controls = []
identity = {"preset": "custom", "delivery_profile": "custom", "lufs_target": None,
            "ceiling_dbtp": 0.0, "bit_depth": 32, "compression_mode": "off",
            "adapt_strength": 0.0}
for k in ("input01", "input06", "input07", "input08"):
    controls.append({"id": f"C3_identity_{k}", "group": "C3", "input": INPUTS[k],
                     "spec": identity, "measure": "light"})
for n in ("pink_-20_60s.wav", "drums_60s.wav", "sine1k_-20dBFS.wav"):
    controls.append({"id": f"C3_identity_{n[:-4]}", "group": "C3", "input": SYN(n),
                     "spec": identity, "measure": "light"})
# identity at 24-bit (dither only)
controls.append({"id": "C3_identity24_input01", "group": "C3", "input": INPUTS["input01"],
                 "spec": dict(identity, bit_depth=24), "measure": "light"})
# C4 gain-only landing
gain_only = dict(identity, lufs_target=-14.0, ceiling_dbtp=-1.0)
for k in ("input01", "input02", "input06", "input07"):
    controls.append({"id": f"C4_gainonly_{k}", "group": "C4", "input": INPUTS[k],
                     "spec": gain_only, "measure": "light"})
# C4b gain-only where the ceiling binds (quiet stem sum to -9)
controls.append({"id": "C4_gainonly_ceilingbound_input07", "group": "C4", "input": INPUTS["input07"],
                 "spec": dict(identity, lufs_target=-9.0, ceiling_dbtp=-1.0), "measure": "light"})
# C5 repeat determinism + R7 VM invariance
for k in ("input02", "input04"):
    controls.append({"id": f"C5_repeat_{k}", "group": "C5", "input": INPUTS[k],
                     "spec": DEFAULT, "measure": "light", "repeat": 2})
    controls.append({"id": f"R7_vm_on_{k}", "group": "R7", "input": INPUTS[k],
                     "spec": dict(DEFAULT, volume_match=True), "measure": "light"})
dump("cases_controls.json", controls)

# ---- edge inputs (R12) ------------------------------------------------------
edge = []
for n in ("silence_10s.wav", "clip_50ms.wav", "clip_2p5s.wav", "square100_0dBFS.wav",
          "isp_+3dBTP.wav", "pink_dc0.2.wav", "noise_-60.wav", "sine19k_-6.wav",
          "burst1k_0dBFS.wav", "pink_wide_corr0.wav"):
    edge.append({"id": f"R12_{n[:-4]}", "group": "R12", "input": SYN(n), "spec": DEFAULT,
                 "measure": "light"})
for n in ("input04_mono.wav", "input04_6ch.wav", "drums_+6dB_overfs.wav"):
    edge.append({"id": f"R12_{n[:-4]}", "group": "R12", "input": VAR(n), "spec": DEFAULT,
                 "measure": "light"})
# loud preset on the hottest synthetic material with the loud-rock profile
edge.append({"id": "R12_drums_loud_loudrock", "group": "R12", "input": SYN("drums_60s.wav"),
             "spec": {"preset": "loud", "intensity": 1.0, "delivery_profile": "loud-rock"},
             "measure": "light"})
edge.append({"id": "R12_square_loud_loudrock", "group": "R12", "input": SYN("square100_0dBFS.wav"),
             "spec": {"preset": "loud", "intensity": 1.0, "delivery_profile": "loud-rock"},
             "measure": "light"})
# CD profile (16-bit, 44.1) and vinyl on the drums
edge.append({"id": "R12_drums_cd", "group": "R12", "input": SYN("drums_60s.wav"),
             "spec": dict(DEFAULT, delivery_profile="cd"), "measure": "light"})
dump("cases_edge.json", edge)

# ---- main music matrix --------------------------------------------------------
main = []
for k, p in INPUTS.items():
    for preset in PRESETS:
        main.append({"id": f"M_{k}_{preset}", "group": "main", "input": p,
                     "spec": dict(DEFAULT, preset=preset), "measure": "full"})
dump("cases_main.json", main)

# ---- sweeps on the subset -----------------------------------------------------
sweeps = []
SUB = ("input02", "input04", "input06")
for k in SUB:
    p = INPUTS[k]
    for prof in ("apple-music", "cd", "vinyl-premaster", "loud-rock", "broadcast-eu", "broadcast-us"):
        sweeps.append({"id": f"S_prof_{k}_{prof}", "group": "profile", "input": p,
                       "spec": dict(DEFAULT, delivery_profile=prof), "measure": "light"})
    sweeps.append({"id": f"S_prof_{k}_custom-9", "group": "profile", "input": p,
                   "spec": dict(DEFAULT, delivery_profile="custom", lufs_target=-9.0,
                                ceiling_dbtp=-1.0, sample_rate=48000, bit_depth=24),
                   "measure": "light"})
    sweeps.append({"id": f"S_prof_{k}_custom-6", "group": "profile", "input": p,
                   "spec": dict(DEFAULT, delivery_profile="custom", lufs_target=-6.0,
                                ceiling_dbtp=-1.0, sample_rate=48000, bit_depth=24),
                   "measure": "light"})
    for inten in (0.0, 0.25, 0.75, 1.0):
        sweeps.append({"id": f"S_int_{k}_{inten}", "group": "intensity", "input": p,
                       "spec": dict(DEFAULT, intensity=inten), "measure": "full"})
    sweeps.append({"id": f"S_comp_{k}_off", "group": "compression", "input": p,
                   "spec": dict(DEFAULT, compression_mode="off"), "measure": "full"})
    sweeps.append({"id": f"S_comp_{k}_density1", "group": "compression", "input": p,
                   "spec": dict(DEFAULT, compression_density=1.0), "measure": "full"})
    sweeps.append({"id": f"S_comp_{k}_density0", "group": "compression", "input": p,
                   "spec": dict(DEFAULT, compression_density=0.0), "measure": "full"})
    sweeps.append({"id": f"S_comp_{k}_manual", "group": "compression", "input": p,
                   "spec": dict(DEFAULT, compression_mode="manual", manual_low_threshold_db=-24.0,
                                manual_low_ratio=3.0, manual_mid_threshold_db=-18.0,
                                manual_mid_ratio=2.0, manual_high_threshold_db=-20.0,
                                manual_high_ratio=2.5, manual_link_stereo=False),
                   "measure": "full"})
    for ad in (0.0, 1.0):
        sweeps.append({"id": f"S_adapt_{k}_{ad}", "group": "adapt", "input": p,
                       "spec": dict(DEFAULT, adapt_strength=ad), "measure": "full"})
    for lvl in ("-6dB", "-12dB"):
        sweeps.append({"id": f"S_level_{k}_{lvl}", "group": "level", "input": VAR(f"{k}_{lvl}.wav"),
                       "spec": DEFAULT, "measure": "full"})
    for rate in ("48k", "96k"):
        sweeps.append({"id": f"S_rate_{k}_{rate}", "group": "rate", "input": VAR(f"{k}_{rate}.wav"),
                       "spec": DEFAULT, "measure": "full"})
for n in ("input04_mono", "input04_dualmono", "input04_swapped"):
    sweeps.append({"id": f"S_chan_{n}", "group": "channels", "input": VAR(f"{n}.wav"),
                   "spec": DEFAULT, "measure": "light"})
# held-out synthetic (rendered for R1-R4 only)
sweeps.append({"id": "H_drums_-6dB_96k", "group": "heldout", "input": VAR("heldout_drums_-6dB_96k.wav"),
               "spec": DEFAULT, "measure": "light"})
dump("cases_sweeps.json", sweeps)

# ---- synthetic tone probes (distortion / aliasing) at default and loud ----------
tones = []
for n in ("sine1k_-20dBFS.wav", "sine1k_-6dBFS.wav", "sine1k_-1dBFS.wav", "imd_smpte_-6.wav",
          "sine15k_-6.wav", "sine19k_-6.wav", "noise_-60.wav", "pink_-20_60s.wav",
          "pink_bright.wav", "pink_boomy.wav", "drums_60s.wav"):
    for preset in ("universal", "loud", "custom"):
        # no landing so that the chain's own gain structure is visible; float out
        tones.append({"id": f"T_{n[:-4]}_{preset}", "group": "tones", "input": SYN(n),
                      "spec": {"preset": preset, "intensity": 0.5, "delivery_profile": "custom",
                               "lufs_target": None, "ceiling_dbtp": -1.0, "bit_depth": 32},
                      "measure": "light"})
dump("cases_tones.json", tones)
