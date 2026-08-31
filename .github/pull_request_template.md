## Slice and gate

- Roadmap item / active `SPEC.md` slice:
- Steering gate closed or explicit exemption:
- Does this change a protected path (`drl-core`, `drl-protocol`, `drl-mcp`,
  `drl-app`, `drl-web`, `drl-script`, or `docs/legacy-behavior`)?

## Observable outcome

<!-- Describe the user-visible or repository-contract result. -->

## Verification

- [ ] `sh scripts/check-repository.sh`
- [ ] `sh scripts/check-version.sh`
- [ ] Focused tests / supported test-play commands:
- [ ] Hosted checks inspected after the final revision:

## Determinism review

For a protected-path change, an independent reviewer must inspect the current
head revision and leave the exact receipt below in the review body. A receipt
for an older commit does not satisfy the policy, and the reviewer must not be
the pull-request author.

`drl-determinism-review: PASS`

Review run / artifact:

## Evidence boundary and follow-up

- Legacy, browser, audiovisual, performance, or cross-version evidence that is
  unavailable remains `NOT_RUN` or `INCONCLUSIVE`.
- Known limitations or follow-up items:
