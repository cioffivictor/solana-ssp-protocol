# The report stability contract (sketch)

A proposal for discussion, and the note I would want to exist before anyone builds on the JSON.

## What is promised, and to whom

The text report is for people, the JSON report is for tools, and the bundle is for both. They do
not have to move together, but they do have to move *deliberately* - which means saying out loud
what a consumer may assume.

## The contract I would propose

1. **Fields are additive.** A new key may appear in a minor release; an existing key is not renamed
   or removed without a major one. A consumer ignores what it does not know.
2. **The verdict is data, not arithmetic.** `outcome` carries the decision. A consumer that
   recomputes "consensus" from the masses will disagree with the engine the first time the policy
   is tuned, and the engine is right by construction.
3. **Numbers are rounded to six decimals and serialised stably.** Two runs over one record produce
   byte-identical reports. That is already true; writing it down makes it a promise rather than an
   accident.
4. **The bundle digest covers the body, not the envelope.** The digest is the drift check; it is
   not a signature, and the note should say so plainly so nobody mistakes it for one.

## What this deliberately leaves out

Timing, ordering across multiple reports, and anything about how a consumer stores history. Those
are the consumer's problems, and pretending otherwise turns this file into a specification for
somebody else's application.

## Open question

Is six decimals the right rounding to freeze? It is what the engine already does, and freezing it
in prose is only honest if we intend to keep it.
