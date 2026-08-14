# Security Policy

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private
[security advisory](https://github.com/haebibisimax/SSP/security/advisories/new)
form, or contact a maintainer directly if the form is unavailable to you.

A useful report includes:

- what an attacker gains, in one sentence,
- the version or commit you are running,
- the smallest reproduction you can manage, and
- whether the input is attacker-controlled in your setup.

You should get an acknowledgement within two working days and a disclosure timeline
before anything is published.

## Scope

SSP adjudicates a *record* of a deliberation; it does not run agents. The
inputs worth attacking are the evidence files it parses and the bundle digests it
verifies. A malformed evidence file that crashes the engine, a claim id that is
accepted when it should be rejected, or a bundle whose digest check can be bypassed
are all in scope.
