# Swift

The Swift package exposes all 33 Pele-compatible diagram families through the
Rust engine. macOS 14+, Swift 6+. The repository root is the consumer package;
its checksum-pinned XCFramework contains arm64 and x86_64 native slices. Build the current native SDK before resolving a local dependency.

```swift
// In your application's Package.swift:
.package(path: "/absolute/path/to/pele-rs")
// Target dependency:
.product(name: "Pele", package: "pele-rs")
```

## Render and retain models

```swift
import Pele

let engine = Pele.Engine()
let document = try engine.parse("pie\n\"Swift\":2\n\"Web\":3")
let drawing = try engine.render(document)
// drawing.type, drawing.svg, drawing.width, drawing.height, drawing.links
```

A `Document` owns its exact native handler, config and parsed model. It remains
renderable after registry replacement. `parseMutable` supplies an editable owned
graph, snapshots, revision-checked `apply`, schema-described `invoke`, and scoped
`bindHostFunction` callbacks. Graph transport preserves aliases, cycles, UTF-16
code units, IEEE number bits and partial state on errors. Close mutable documents
when finished. See [mutable APIs](../../docs/MUTABLE_MODELS.md).

`parsePrepared("pie", source: ...)` exposes the low-level family parser without
core source preparation. `createPrepared("pie", graph: ...)` accepts a caller's
model graph without parsing source. Each returns a separate `PreparedDocument`
with snapshot/apply/schema/invoke/render/close ownership. All 33 families expose
these entries; declared models retain upstream missing-helper failures rather
than receiving invented parser state. See
[native entries](../../docs/NATIVE_DIAGRAM_ENTRIES.md) and
[construction](../../docs/DECLARED_MODEL_CONSTRUCTION.md).

## Native presentation and resources

`Pele.DiagramView`, `Pele.mount`, `Pele.Mounted` and `Pele.enableZoom` provide the
AppKit lifecycle. Parsing and geometry execute in Rust; AppKit owns drawing,
input, resource approval and accessibility controls.

```swift
@MainActor
func show(in view: Pele.DiagramView) throws -> Pele.Mounted {
    let mounted = try Pele.mount(view, source: "flowchart LR\nA[Swift] --> B[Rust]")
    // Retain mounted for update(), invalidateResources() and destroy().
    return mounted
}
```

Mounts retain the last good drawing on failure, observe size/font changes, and
support explicit updates after automatic observation is destroyed. Native zoom
handles mouse, wheel, magnification, keyboard and controls without rerendering.
Links use Rust-produced action regions with pointer/keyboard activation and
accessibility actions. Native GUI verification covers these controls; it does
not claim physical-trackpad or spoken VoiceOver testing.

A mount owns `Pele.NativeResources` by default. It measures installed fonts through
CoreText, keeps bounded font/image caches, and requires host approval and supplied
PNG/JPEG bytes for image references. It does not load files or fetch URLs itself.
Use `MountOptions.nativeResources` to share an explicit owner, or call
`Engine.renderNative(_:nativeResources:)` directly on MainActor. Missing fonts or
unapproved images fail explicitly. `invalidateResources()` clears the generation
and redraws. Ordinary `HostResources` callbacks remain available for custom
measurement, icons, clocks and calendars.

`Pele.Registry.empty()` provides an owned selective registry with independent
registration/loading and retained document lifetimes. Async mounting selects
families before rendering; it does not download native code. `Pele.AsyncEngine`
is an independent actor-owned rendering API. AppKit presentation runs on MainActor.

## Build and verify from source

Build the current SDK before resolving a local package dependency:

```sh
scripts/build-native.sh                    # Host architecture
scripts/build-native.sh --universal        # macOS arm64 + x86_64
swift test --package-path bindings/swift --skip 'MountTests|ZoomTests|PresentationUITests|NativeResourceUITests'
scripts/test-swift-ui.sh                   # Active macOS desktop required
swift run --package-path bindings/swift pele-smoke "$PWD" "$PWD/dist/evidence"
swift run --package-path tests/swift-consumer
```

The local artifact at `bindings/swift/Artifacts/CPele.xcframework` selects the
source-built SDK. It must match the Swift sources and C header. Final acceptance
passes 176 CLI features and 28 strict C executions across both architectures,
all-33-family pixel/consumer checks on each, and 31 active AppKit features. See
[verification evidence](../../docs/FULL_PARITY_PROGRESS.md) for exact scope,
Rosetta discovery recovery and artifact boundaries.

No Kamui dependency, web view or JavaScript diagram renderer is required.
No iOS or Windows binary is included. Native transport/work budgets and
documented compatibility boundaries remain active.

For local Swift correctness and performance iteration, `scripts/build-native.sh --host` builds an XCFramework containing only the current explicit Rust host target. It matches the standalone Swift consumer freshness check without compiling duplicate untargeted or foreign-architecture archives. Use `--universal` for release validation and distributable macOS artifacts.
