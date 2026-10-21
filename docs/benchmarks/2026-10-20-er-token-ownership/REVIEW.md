# Borrowed-token safety review

Candidate is a coherent two-file ER-only representation layer. All nonempty text comes from the original source slice in `Scanner::emit`; explicit ERROR is empty. No scanner action transforms token payloads. `Reader::take` preserves owned return values and all target signatures, so model lifetime and public APIs do not change. Original source, token kinds, offsets and error formatting remain exact.

The existing source outlives Scanner and Reader. Explicit token/source lifetimes prevent accidental escape without unsafe code. Allocation timing shifts from eager lexical payload creation to owned grammar consumption; strings consumed by the parser still materialize once. Skipped tokens and unconsumed suffixes avoid owned text. Token storage remains a full vector and fallback still performs another complete scan.

No correctness blocker found by static review. Compilation, integration results and target layout are unverified; no executable was built or run. Source-derived counts are potential logical String materializations removed, not measured allocator or timing results. Keep this separate from parser grammar, fallback handoff, direct host-store construction and other parsers.
