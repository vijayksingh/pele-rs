# Prepared ER host publication

Source `80bed95`, including the preceding typed admission-count transport.
Unreleased. Prepared/standalone ER uses its captured prepared grammar once,
then counting admission and eager host publication. The transient typed model
drops before return. Low-level NativeRegistry/MutableDocument arena contracts
and custom provider paths remain unchanged.

The six previously failing prepared callback comparisons now pass in Chromium,
Firefox and WebKit, without changing assertions. Full SVG/dimension bits, traces,
state, identities, original throws and nested raw rendering are compared.

Verification:

- [39 ER-focused browser checks](web.log.gz), followed by
  [21 prepared/all-33-entry checks](entries-web.log.gz), with overlap. This includes
  old-arena normal-render policy, captured config/providers, lifecycle, thirteen
  low-level error cases and unchanged binary document APIs. Detailed reports
  accompany the second run.
- [Twelve native integration tests](native-retry.log.gz): five new prepared
  publication/provider/budget tests, three fallback and four registry tests.
  The initial test harness had two invalid `Object::new` calls; replacing them
  with `Object::default` fixed compilation. The initial log remains archived.
- Fresh ARM64 SDK: [168 Swift API conditions](swift-api.json),
  [eight ER/eager integrations](swift-corpus.log.gz), and
  [seven prepared/caller-model integrations](swift-prepared.log.gz), including
  all 33 families' raw and caller-built model fixtures. [Three C consumers](c.log.gz),
  TypeScript, formatting and diff checks pass.

The [12-condition independent VM diagnostic](first-comparison.json) compares
the combined prepared/count build against the frozen inline-host checkpoint,
with nine balanced trials and 500 ms warmup. It does not isolate count-import
causality, compare against the old `0b32797` backend, or replace the full gate.

| Retained ER condition | Paired change | Faster pairs | Ratio of medians | WASM / JS |
| --- | ---: | ---: | ---: | ---: |
| Medium/fixed | -4.12% | 8/9 | -13.86% | 6.00× |
| Medium/Canvas | -4.64% | 7/9 | -4.50% | 5.17× |
| Large/fixed | -6.32% | 7/9 | -6.58% | 5.63× |
| Large/Canvas | -7.59% | 9/9 | -7.25% | 5.12× |

All parse and source controls remain in the [paired summary](paired-summary.json).
They are mixed; separate medians and paired ratios differ substantially in some
rows. No broad speed or confidence claim follows. The candidate's complete
106-file package map matches [the source/SDK freeze](source-freeze.json).
No new local timing claim is made for this layer; local correctness is current,
and earlier local timings remain scoped to their preceding source checkpoint.

Browser caller-authored raw adoption is still a separate gap. Its
[public reproducer](raw-baseline/reproducer.spec.js) fails six callback cases in
each engine while all ten Array/Map/Set/Date/Error root controls pass. Raw rows
are preserved for [Chromium](raw-baseline/chromium.json.gz),
[Firefox](raw-baseline/firefox.json.gz) and [WebKit](raw-baseline/webkit.json.gz).
Original callback throw identity already passes; state and subsequent rendering
still differ. These controls prevent narrowing accepted root kinds during the
next fix. [Evidence index](evidence-index.json).

Full API and JS performance parity remain open. No 0.3 release.
