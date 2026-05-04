# Product

## Register

product

## Users

Primary user is the owner of a MacBook Air M2 who wants to understand system health without opening Terminal or Activity Monitor. The usage context is quick diagnosis during daily work: the user wants to know what is slowing, heating, or draining the machine, and whether an app can be closed safely.

## Product Purpose

Mac Health Monitor is a lightweight native macOS dashboard for real-time local health telemetry. It surfaces CPU, memory, storage, network, battery, thermal state, Rosetta candidates, and long-window process consumption in a calm, actionable interface.

Success means the user can identify the current bottleneck and the top sustained offenders within seconds, without noisy raw telemetry or fake precision. The app should feel fast, native, trustworthy, and easy to leave open.

## Brand Personality

Calm, precise, premium.

The product should borrow the speed and focus of Raycast, the hierarchy discipline of Linear, and the diagnostic credibility of iStat Menus, while using a warm light surface rather than a dark technical cockpit.

## Anti-references

Do not imitate a fake macOS window chrome. Do not create a Windows-style dashboard clone. Do not stack dense metrics in the same visual zone. Do not use heavy animations, decorative clutter, or chart-heavy panels that make process diagnosis harder.

## Design Principles

1. Diagnose before decorating: every visual decision should help the user understand health, load, or actionability faster.
2. Average beats jitter: sustained 5-minute process ranking should be visually calmer and more important than momentary spikes.
3. Soft but exact: use warm, comfortable materials while keeping numbers, labels, and warning states crisp.
4. Make the culprit obvious: if one app is slowing, heating, or consuming memory, the interface should make that app easy to spot.
5. Stay lightweight: avoid dependencies, oversized assets, expensive effects, and interactions that would undermine the purpose of a performance monitor.

## Accessibility & Inclusion

Target WCAG AA contrast for text and controls. Do not rely on color alone for health states. Respect reduced motion preferences. Keep controls reachable with keyboard focus states and maintain readable type at small laptop sizes.
