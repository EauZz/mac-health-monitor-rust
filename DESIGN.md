# Design

## Visual Theme

Warm diagnostic workspace. The interface should feel like a quiet instrument panel on a cream desk surface: light, focused, premium, and operational without becoming sterile.

## Color Palette

- Canvas: warm cream, ivory, and light sand gradients.
- Panels: translucent warm white with subtle borders and soft depth.
- Ink: near-black graphite for primary text.
- Muted text: slate-taupe for secondary labels.
- Primary accent: electric blue for CPU, network, focus, and primary interactions.
- Health accent: leaf green for battery, good status, and stable states.
- Warning accent: amber for attention and heat caution.
- Critical accent: red/coral for sustained heavy process load or urgent states.
- Memory accent: restrained violet only when useful for RAM distinction.

## Typography

Use native Apple system fonts for performance and macOS fit. Prefer a strong hierarchy through size, weight, spacing, and measure rather than external font loading.

- Page title: compact, confident, not oversized.
- Card titles: short, high-contrast, immediately scannable.
- Metric values: tabular, prominent, calm.
- Labels and hints: concise French copy with low visual noise.

## Layout

The default layout is a full-window product surface, not a fake app window. Use a max-width content rail for wide screens, with generous but efficient spacing.

Priority order:

1. Health summary and global actions.
2. CPU, memory, thermal, and process watch.
3. Battery, storage, network, and system details.
4. Secondary status footer.

Process Watch should use a focused, single active list rather than multiple expanded lists. Top offenders should have strong rank, name, metric, and one short action hint.

## Components

- Soft metric cards with subtle glass, but no heavy blur dependence.
- Segmented controls for process categories.
- Compact ranked rows for sustained offenders.
- Mini charts as supporting texture, not primary content.
- Health chips and badges with text plus color.
- Footer should be quieter than the main diagnostic area.

## Motion

Motion should be minimal and functional: short fades, subtle value transitions, tab state transitions, and chart updates. Avoid cinematic effects. Respect `prefers-reduced-motion`.

## Responsive Behavior

Desktop and laptop are primary. The interface should still stack cleanly on narrow windows with no horizontal overflow. Process Watch must remain readable when cards collapse to one column.

## Implementation Notes

Use the existing HTML/CSS/vanilla JS stack. Do not add a frontend framework or runtime dependency. Keep charts on canvas and keep the native WKWebView app lightweight.
