# Independent projection-policy review

No blocking static finding. The one-file change preserves the projection contract while removing stored policy bytes from unit builtin modules. Compilation, integration execution, allocation capture execution and any elapsed-time comparisons remain the parent's work; this reviewer did not build, run tests/consumers, or edit production.

Reviewed patch SHA256 `fd2d80ccad49213125801990bbc7093557869e3efc8aadaceb77560d6e8d0405`. Original engine SHA `ff3e2806feec7c1ffe541cb5ca86559dc238a53c56a1ac65cd6b66dd2ac54cae`; candidate/applied SHA `9805911042d8fac20d3343bdeba5ae720a4629c3089aa3dcd5628bcae61ce1b2`. Manifest, originals, private candidate, patch and applied source all matched during review.

## Type and ownership semantics

`Module<D, P>(D, PhantomData<P>)` stores the module and a zero-sized marker. Both policies are unit structs; PhantomData has size zero and alignment one. With the existing unit builtin D, the concrete Module is therefore zero-sized and Box::new requires no backing heap allocation for that value. Converting that box to `Box<dyn ErasedModule>` adds the vtable metadata without allocating a separate trait-object container. The outer Arc registration, registry vector and stateful provider storage still allocate normally; this is not an allocation-free registry claim.

`DiagramModule` already requires Send + Sync + 'static. `ModelProjection` explicitly requires the same; the PhantomData field carries P's relevant auto-trait/lifetime constraints rather than bypassing them. The private policy trait's generic method need not be object-safe because the policy is used statically. The actual erased boundary remains the unchanged object-safe ErasedModule. No new unsafe code or public ABI/layout dependency is introduced.

Public `register` always creates `Module<D, Legacy>`. Only the concrete builtin Gantt registration selects NormalizedBuiltin. The existing Arc retains the exact module object and its concrete vtable, which now selects the policy through monomorphization. Replacing a same-ID provider, dropping the Engine or separately registering Gantt through the public API cannot change the policy of a captured document. The model's Any downcast remains tied to D, independent of policy. Stateful providers keep their state and normal drop behavior.

`model_view` still downcasts, validates exactly once, then calls the same projection implementation. Public providers retain one Serialize invocation through legacy project_model. Normalized Gantt retains its existing pure-builtin exceptional fallback; this patch neither adds retries nor changes byte/depth/error precedence. Parse, query, render, config and document ownership are unchanged. Separate policy monomorphizations can affect code layout, so source reasoning alone cannot establish an elapsed-time improvement or explain the earlier VM source-render observation.

## Registry allocation diagnostic

Reviewed `crates/pele/examples/registry_allocations.rs`, source SHA `c40c77ee14a14af9105834aba2e4542109fe8ca16ef06dbbb310d68e80968d91`. The frozen control binary matches `c2cd0a31a563c0a1f8fa0d759d2868437b66e5aa805fed967cfe71078d84a243`; all three recorded source hashes match the appropriate baseline/current copies. Its manifest records the instrumented native build command and revision d55b961. It is a focused three-source manifest, not a complete source/dependency/toolchain aggregate like the Swift consumer manifest.

The diagnostic correctly brackets only public `Engine::with_builtins` construction after one untimed/uncounted warmup. It resets counters while disabled, enables before construction, disables before capability inspection and destruction, and allocates the report outside capture. Black-boxing keeps the constructed value observable. The allocator wrapper does not format or allocate while recording. It forwards each allocation/deallocation/reallocation to System and records only successful calls. There is no elapsed-time or RSS collection.

Interpret counters precisely:

- `allocations` counts successful alloc and alloc_zeroed calls, excluding realloc.
- `oneByteAllocations` counts one-byte requests in those allocation calls, not realloc requests or allocator-reserved block sizes.
- `reallocations` is separate; `requestedBytes` sums fresh allocation request sizes plus the **entire new size** of each successful realloc. It is cumulative requested traffic, not net growth, peak live bytes, retained bytes or physical memory saved.
- Deallocation is deliberately not counted; the engine is dropped after capture. The global enable flag counts process-wide allocator activity, although this constructor/example introduces no worker threads. Results describe this optimized instrumented native executable, not Swift or WASM allocator behavior.

The saved baseline contains five identical rows: **69 allocations, 33 one-byte allocations, 4 reallocations, 3,948 requested bytes**. After the initial static review, the parent captured `candidate-registry-allocations.json`; independently rereading it confirms five identical candidate rows: **36 allocations, zero one-byte allocations, 4 reallocations, 3,915 requested bytes**. The measured difference is exactly **33 fewer allocation calls, all one-byte requests; reallocations unchanged; 33 fewer cumulative requested bytes**, matching the earlier source-derived prediction. The reviewer did not execute either binary. Candidate report SHA256: `c59158ea14a1b228226b16e44c50853f8a73b079fa630fcdd8d20bd44da7a199`.

The observed request-count reduction is direct evidence in this instrumented native constructor, consistent with removal of the 33 policy-containing boxes. It does not establish elapsed-time improvement. The 33 requested-byte reduction must not be presented as 33 bytes of actual heap/RSS savings: allocator size classes and overhead differ.

Existing public normalized-projection integrations already exercise registration replacement, captured documents, provider Serialize-once and validation/error behavior. Reusing those feature tests is proportionate; an implementation-only size test is unnecessary. The parent additionally reports 20 native feature integrations passed; those executions were not independently rerun here. Any timing claim requires the separate matched delivery-lane comparison.
