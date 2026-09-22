# YES Master pricing assessment — September 14, 2026

Status: researched recommendation, not adopted pricing or release approval.
The owner explicitly reopened the former $29 founder / $49 standard figures:
"those prices arent written in stone". The request is to value the actual
product against competing software and the trust considerations of a new developer.

## Recommendation

Keep the planned eight-week public beta free. For a qualified paid 1.0, test
**$49 introductory / $79 standard**, one perpetual desktop SKU with Standard,
Advanced and Album included. These are proposed prices, not new public terms.
The introductory window remains undecided. Treat the existing $29/$49 figures
as the prior hypothesis rather than a constraint on this assessment.

$49–$79 is the most defensible first-release range in my judgment. $99 could
be viable with stronger independent listening evidence and demonstrated workflow
preference; $149+ is not supported by the present evidence. This is a commercial
hypothesis, not measured willingness to pay or proof of sound superiority.

The product's useful unit of value is a finished track or album with a clear,
repeatable workflow. Development effort and the number of controls do not by
themselves determine customer value. A focused standalone application can justify
more than a single plugin, but established standalone competitors limit the premium.

## What is being valued

Inspected current PRODUCT.md, ARCHITECTURE.md, APP_BEHAVIOR.md, the capability
matrix, current decision notes, installed evidence, export-format source, the
browser worker and desktop packaging configuration. This was a source/document
assessment; no fresh listening comparison or native qualification was performed.

| Current capability | Potential customer value | Limit on the claim |
| --- | --- | --- |
| Standard workflow with styles, intensity and loudness choices | Short path from an existing mix to a master | Ease relative to competitors has not been measured with new users |
| Advanced EQ, multiband compression, transient shaping, width, warmth and limiting | One purchase can serve users as they want more control | Feature presence does not prove equal or better sound than other engines |
| Original/Mastered switching with retained position and optional audition-only Volume Match | Hear changes fairly without losing the listening position | Not unique across mastering products |
| Active source-profile restraint and measured loudness landing | Processing responds to the material within defined bounds | Not reference matching or a generally trained AI mastering model |
| Album organization, overrides, batch and continuous exports | Finish an EP/album without manually rebuilding each delivery workflow | No DDP, cue sheets, ISRC or general gapless-playback guarantee |
| WAV, MP3, FLAC, AIFF, M4A, ADTS AAC and Ogg candidate exports | Useful delivery coverage in the mastering workflow | Windows candidate evidence does not certify every future Mac/release artifact |
| Export receipts, decoded-output checks and source protection | Know what file was delivered and avoid preventable delivery mistakes | Measurements do not certify that a master sounds good |
| Local native processing and a standalone app | No DAW needed; no audio upload needed for desktop mastering | No delivered VST3/AU/AAX product was identified |

Exclude disabled adaptive-compressor/Phase-B/album-character work, deferred
mobile launches and hypothetical paid browser export from the paid-v1 valuation.
The browser audition is currently a 30-second core-chain preview, without full
desktop source-aware behavior or export.

The current branch's September 14 decision ledger still records unresolved release
qualification, including Intel Mac Album PCM comparison evidence. The September 9
installed Windows owner PASS remains valid for its scope. This assessment did not
refresh remote CI and does not make a new release-readiness verdict. A cheaper
price is not a substitute for resolving known delivery defects.

## Competitive offers checked

Observed September 14, 2026. USD unless a euro price is explicitly shown.
Prices are offers/listings, not a guarantee of final tax-inclusive checkout.
Full licenses are distinguished from upgrades, crossgrades and education variants.

| Product | Observed offer | Relevant comparison |
| --- | --- | --- |
| [HoRNet MasterTool](https://www.hornetplugins.com/plugins/hornet-mastertool/) | €22.99 | Automated EQ/dynamics mastering plugin; evidence that automatic processing alone is inexpensive |
| [Brainworx bx_masterdesk PRO](https://www.plugin-alliance.com/products/brainworx-bx_masterdesk-pro) | $49.99; comparison price $175 | Integrated mastering chain with dynamics, EQ, saturation, stereo controls and true-peak limiting; plugin workflow |
| [Ozone 12 Elements](https://www.izotope.com/products/ozone-elements) | $55 full license | Master Assistant and assistive vocal balance; individual processing modules are not accessible |
| [TDR Limiter 6 GE](https://www.tokyodawn.net/tdr-limiter6-ge/) | €60, EU VAT where applicable | Dynamics/limiting toolkit and loudness metering; a narrower component, not a full file/album application |
| [Lurssen Mastering Console Mac/PC](https://www.ikmultimedia.com/shop/cart.php?alias=lurssen-macpc) | $99.99 full license | Focused mastering software with an established engineering pedigree |
| [T-RackS 6](https://www.ikmultimedia.com/shop/cart.php?alias=t-racks-6) | $99.99 base edition | 19 processors, plugin and mastering console; do not attribute the larger editions' whole catalogue to this price |
| [Ozone 12 Standard](https://www.izotope.com/products/ozone-standard) | $219 full license | 14 editable modules, referencing and deeper DAW mastering workflow |

T-RackS also explicitly supports a standalone track/album workflow, so standalone
operation is useful differentiation from plugins, not a unique category invention.
[Official T-RackS overview](https://www.ikmultimedia.com/products/tr6/).

The official indexed [WaveLab Elements 13 listing](https://www.steinberg.net/wavelab/elements/)
showed $99.99, with $39.99 update/upgrade options separately identified. The live
text fetch returned a JavaScript shell, so this price has lower freshness confidence
than the directly retrieved store offers above. It is supporting context, not a
decisive exact-price anchor.

Free alternatives matter as well: [BandLab](https://help.bandlab.com/hc/en-us/articles/360001374513-Using-BandLab-Mastering-on-your-songs)
offers four free mastering presets; [Matchering](https://github.com/sergree/matchering)
is open-source reference-based matching/mastering with desktop and self-hosted
options. Setup and workflow differ. No independent sonic ranking is inferred
from their developers' descriptions or marketing quotations.

### Price verification details

- Plugin Alliance's public first-party product data at
  `https://www.plugin-alliance.com/products/brainworx-bx_masterdesk-pro.js`
  returned available=true, price=4999, compare_at_price=17500, with the storefront
  identifying United States/USD. Use $49.99, not an older indexed $29.99 offer.
- iZotope's product page showed Ozone Standard at $219. Its first-party
  `https://www.izotope.com/products/ozone-standard.js` returned the ordinary
  full-license variant at 21900. The product-level minimum of 12900 belongs to
  an update variant; it is not the price for a new customer.
- HoRNet, IK and Ozone Elements prices were retrieved from their product/store
  pages. No account-specific offer, coupon, cart change or purchase was used.

## New developer considerations

The relevant adjustment is for buyer uncertainty, not the developer's seniority.
Buyers need confidence that the audio sounds good on their material, the product
works on their computer, exports are dependable, and support/compatibility will
continue. A new name has less public evidence for those expectations.

Reduce that uncertainty with a useful trial, clear support contact, accurate
compatibility information, a considered refund policy and independent examples
that include level-matched comparisons. The browser excerpt should not be
presented as an exact full-track export audition. Technical regression coverage
supports reliability but is not a substitute for listeners or customer adoption.

New-developer status does not automatically justify a permanent budget price.
Very low revenue per customer can leave less capacity for the support and updates
that establish trust. Conversely, a higher price does not manufacture that trust.

## Why $49 introductory / $79 standard

- $49 lowers the initial commitment while maintaining a meaningful amount per
  customer. It sits near current entry-level/plugin offers, with YES Master's
  complete standalone track/album workflow as the reason to choose it.
- $79 asks a modest premium over those plugins, while remaining below the
  observed $99.99 established standalone offers. The pitch must emphasize an
  easier route to a finished master, not an unsupported claim to beat them.
- $99 would invite a more demanding comparison with T-RackS and Lurssen. It
  could work if users demonstrably prefer YES Master's results/workflow, but
  that evidence is not available in this assessment.
- $149+ requires a stronger reason to switch and greater market confidence than
  the current evidence provides. Adding controls just to reach a price is not
  recommended.
- Keep one coherent SKU initially. Recommend perpetual use of the purchased
  major version and maintenance within it; separately priced future major
  upgrades can be optional. Do not promise every future release forever as a
  side effect of the word "perpetual". These are proposed terms only.

## Economics and validation

Using Lemon Squeezy's published base fee of 5% + $0.50, hypothetical receipts
are $27.05 on $29, $46.05 on $49, $74.55 on $79 and $93.55 on $99.
Additional fees may apply. These exclude acquisition, support, refunds,
infrastructure and other costs. [Published fee schedule](https://www.lemonsqueezy.com/pricing).

At equivalent traffic and otherwise equal costs, $79 needs about 61.8% as many
purchases as $49 to match those receipts. This is break-even arithmetic, not
an expected conversion improvement. Customer lifetime support and paid-upgrade
revenue remain unknown.

Use the existing beta to observe completion of the actual import/audition/export
workflow and collect voluntary feedback. Do not add installed-app telemetry or
claim existing conversion/retention data. Then evaluate actual paid offers with
comparable traffic, monitoring net revenue per visitor, refund rates and support
effort as well as conversion. Small cohorts yield directional evidence, not
precise elasticity estimates. Survey answers alone do not establish willingness
to pay. No price test or data collection is activated by this recommendation.

## Effect on the browser-versus-software decision

Reopening desktop pricing weakens the earlier argument based on a $49 license.
At $79 versus a hypothetical $5 per track, the consumer spend crossover is about
16 tracks. A higher desktop price leaves more room for an occasional-use offer.

I still recommend software-only monetization for the first paid release, with
free browser audition supporting discovery and evaluation. The deciding factors
are proving demand, earning trust and supporting one dependable paid workflow.
Neither the free beta nor an interesting demo establishes incremental demand for
paid browser delivery. Revisit that offer if customers demonstrably want to pay
for an online finished file but will not install the desktop application.

## Follow-up: mechanical comparison, a phone bundle and the meaning of $49

The owner subsequently proposed one desktop experience plus one mobile experience
for one purchase price, and asked whether mechanical audio comparisons are possible
and whether $49 can make the product feel cheap. This is a bundle direction under
discussion, not a newly adopted device limit, price or simultaneous release date.

### Mechanical audio comparison

The existing `src-tauri/examples/private_reference_tuning.rs` calls
`run_reference_tuning_dir` in `src-tauri/src/reference_tuning.rs`. It renders
YES Master against supplied external masters and writes JSON/CSV comparison
ledgers. Current fields include LUFS, dynamics, low/mid/high spectral differences,
transient density, width/correlation and warning codes. It expects a specific
source/four-preset naming convention; it is not a general competitor host or an
independent perceptual-quality benchmark. No new comparison was run for this reply.

Recommended extension:

1. Use the same lossless input in each product; record versions, settings, source
   hashes, format and processing path. Include a varied music corpus and separate
   synthetic stress signals. The old tuning source must not be the entire benchmark;
   include material not used to tune YES Master.
2. Produce both ordinary default-workflow renders and a controlled matched-target
   comparison where settings permit it. Do not force unlike controls into false
   equivalence. Preserve original delivered files for objective measurements.
3. Independently decode/measure loudness, true peak, clipping/non-finite values,
   duration and channel integrity. Add frequency/temporal/stereo descriptions on
   music and aliasing/distortion/limiter-response probes under appropriate synthetic
   conditions. Intentional saturation and dynamic shaping are not automatic defects.
4. Generate aligned listening copies at a common loudness with enough headroom,
   randomized labels and source excerpts. ABX assesses distinguishability; a separate
   blind preference exercise assesses which master listeners prefer.

Loudness and true peak have a defined measurement basis in
[ITU-R BS.1770](https://www.itu.int/rec/R-REC-BS.1770/en).
Reference similarity, a null-test residual or a set of matching summary statistics
does not establish equal sound quality. A reference master is a comparison target,
not ground truth. Do not collapse the report into a purported universal quality
score or claim that mastering is better merely because it changes the source less.

### Desktop plus phone

Both `apps/iphone-native` and `apps/android-native` contain real phone implementations
around the shared Rust engine. The iPhone overview and actual Android controls/share
code support the focused Standard workflow. Historical June shippability plans
contain already-addressed items and must be refreshed before using them as a live
implementation queue. A shared engine alone does not certify current device or
store readiness, and a completed shared purchase/restore system was not established.

The proposed product package is coherent: desktop Standard/Advanced/Album, plus
focused phone import, style/intensity/loudness, A/B and export. Interpret "one
desktop and one mobile experience" as a product bundle, not an invented strict
one-machine activation rule. Cross-device project sync and every future upgrade
are not implicit entitlements.

Common ownership needs an explicit purchase/restore design. Apple's multiplatform
rule permits access to features acquired elsewhere subject to its in-app purchase
conditions; Google Play describes consumption-only access and billing exceptions.
Storefront-specific rules must be checked when implementing. This is feasible
product direction, not evidence of an approved store configuration.
[Apple guidelines](https://developer.apple.com/app-store/review/guidelines/#multiplatform-services),
[Google Play payments guidance](https://support.google.com/googleplay/android-developer/answer/10281818?hl=en).

### Price perception

The general association between price and perceived quality is supported by the
[Rao/Monroe meta-analysis](https://journals.sagepub.com/doi/10.1177/002224378902600309).
It does not establish a $49 threshold or predict reactions to this audio product.
My judgment: $49 is credible as an introductory purchase; a permanent $49 price
for a mature desktop-plus-phone bundle may undersell its scope and leave less
support revenue. Presentation, a convincing trial and actual customer outcomes
matter alongside the number. A higher price cannot manufacture confidence.

The bundle strengthens the case for $79 standard / $49 introductory once both
included experiences are dependable. $99 remains a plausible later test if users
value the phone experience and independent results support it. Do not simply add
two app prices together or inflate a comparison price. The paid offer should state
the mobile scope and actual availability clearly. No price or launch change is
activated by this follow-up assessment.
