import Foundation
import CPele

private struct PublicDocumentMetadata: Decodable {
    let type: String
    let revision: String
}
func ownedPublicDocument(_ result: OpaquePointer?, document: OpaquePointer?) throws -> Pele.MutableDocument {
    do {
        let metadata = try Pele.compatibilityResult(result, as: PublicDocumentMetadata.self)
        guard let handle = document, let revision = UInt64(metadata.revision), String(revision) == metadata.revision else {
            throw PeleError(code: "internal", message: "Invalid mutable document metadata")
        }
        return Pele.MutableDocument(handle: handle, type: metadata.type, revision: revision)
    } catch {
        if let document { pele_public_document_free_v4(document) }
        throw error
    }
}
private struct AppliedRevision: Decodable { let revision: String }
extension Pele {
    /// Owned mutable Rust model. Confine to one owner/actor; this class is not Sendable.
    /// Binary graph crossings are bounded to 64 MiB. Source convenience accepts Swift String;
    /// graph strings preserve isolated UTF-16 units, whose rendering remains codec-dependent.
    public final class MutableDocument {
        private var handle: OpaquePointer?
        private var active = false
        private var detachedDepth = 0
        var hostFunctions: [UInt32: HostFunctionBox] = [:]
        public let type: String
        public private(set) var revision: UInt64
        fileprivate init(handle: OpaquePointer, type: String, revision: UInt64) {
            self.handle = handle; self.type = type; self.revision = revision
        }
        deinit { if let handle { pele_public_document_free_v4(handle) } }
        public func close() throws {
            guard !active && detachedDepth == 0 else { throw PeleError(code: "semantic", message: "Mutable document is already in use") }
            if let handle {
                active = true; self.handle = nil
                defer { active = false }
                pele_public_document_free_v4(handle); hostFunctions.removeAll()
            }
        }
        func withHandle<T>(_ operation: (OpaquePointer) throws -> T) throws -> T {
            guard let handle else { throw PeleError(code: "semantic", message: "Mutable document is closed") }
            guard !active else { throw PeleError(code: "semantic", message: "Mutable document is already in use") }
            active = true; defer { active = false }
            return try withHostFunctionErrorScope {
                try withExtendedLifetime(self) { try operation(handle) }
            }
        }
        // Retain ownership across callbacks while allowing short nested native calls.
        // Disposal remains forbidden for the complete detached lifecycle.
        func withDetachedLifetime<T>(_ operation: (OpaquePointer) throws -> T) throws -> T {
            guard let handle else { throw PeleError(code: "semantic", message: "Mutable document is closed") }
            guard !active else { throw PeleError(code: "semantic", message: "Mutable document is already in use") }
            detachedDepth += 1; defer { detachedDepth -= 1 }
            return try withHostFunctionErrorScope {
                try withExtendedLifetime(self) { try operation(handle) }
            }
        }
        internal func acceptPreparedRevision(_ revision: UInt64) { self.revision = revision }
        public func snapshot() throws -> PublicSnapshot {
            try withHandle { handle in
                struct Metadata: Decodable { let type: String }
                let exchange = try Pele.binaryResult(pele_public_snapshot_binary_v5(handle))
                let metadata = try JSONDecoder().decode(Metadata.self, from: exchange.metadata)
                let snapshot = PublicSnapshot(type: metadata.type, revision: exchange.revision, graph: exchange.graph)
                guard snapshot.type == type else { throw PeleError(code: "internal", message: "Mutable document type changed") }
                revision = snapshot.revision; return snapshot
            }
        }
        /// Explicit full-graph edit boundary. Stale revisions leave the native state unchanged.
        @discardableResult
        public func apply(_ graph: PublicGraph, expectedRevision: UInt64) throws -> UInt64 {
            let bytes = try graph.binaryBytes(revision: expectedRevision)
            guard bytes.count <= 64 * 1024 * 1024 else { throw PeleError(code: "limit", message: "Public binary graph exceeds transport budget") }
            return try withHandle { handle in
                let result = bytes.withUnsafeBytes { input in
                    pele_public_apply_binary_v5(handle, input.bindMemory(to: UInt8.self).baseAddress, input.count)
                }
                let applied = try Pele.compatibilityResult(result, as: AppliedRevision.self)
                guard let next = UInt64(applied.revision), String(next) == applied.revision else {
                    throw PeleError(code: "internal", message: "Invalid mutable revision")
                }
                revision = next; return next
            }
        }
        public func schema() throws -> PublicModelSchema {
            try withHandle { try Pele.compatibilityResult(pele_public_schema_v4($0), as: PublicModelSchema.self) }
        }
        /// Apply graph edits first when needed. Operation errors retain the graph
        /// and revision in the returned outcome. Preflight or transport failure
        /// throws; post-call export failure can leave native state advanced.
        public func invoke(method: UInt32, receiver: PublicGraph.Atom, arguments: [PublicGraph.Atom] = [], resources: HostResources = .init()) throws -> PublicInvocationOutcome {
            struct Call: Encodable { let method: UInt32; let receiver: PublicGraph.Atom; let arguments: [PublicGraph.Atom] }
            let bytes = try JSONEncoder().encode(Call(method: method, receiver: receiver, arguments: arguments))
            guard bytes.count <= 64 * 1024 * 1024 else { throw PeleError(code: "limit", message: "Public method exceeds 64 MiB JSON budget") }
            return try withHandle { handle in
                let response = resources.withCapturedNativeResources { host, takeError in withHostFunctionResources(takeError) { bytes.withUnsafeBytes { input in
                    pele_public_invoke_with_functions_v12(handle, revision, input.bindMemory(to: UInt8.self).baseAddress, input.count, host)
                }}}
                defer { clearHostFunctionErrors() }
                do {
                    let exchange = try Pele.binaryResult(response.handle)
                    let outcome = PublicInvocationOutcome(revision: exchange.revision, graph: exchange.graph, result: try JSONDecoder().decode(PublicOperationResult.self, from: exchange.metadata))
                    revision = outcome.revision
                    if case .thrown(.hostException(let token)) = outcome.result, let original = hostFunctions[token]?.error { throw original }
                    if let error = response.error { throw error }
                    return outcome
                } catch { if let original = response.error { throw original }; throw error }
            }
        }
        /// Stateful native rendering publishes graph/revision even on operation error.
        public func renderOperation(options: Engine.Options = .init(), resources: HostResources = .init()) throws -> PublicRenderOutcome {
            try renderOperation(options: options, resources: resources, outputLimit: nil)
        }
        /// Exact IEEE threshold overrides only options.outputLimit. Callbacks and
        /// renderer mutations precede comparison; operation errors retain state.
        public func renderOperation(outputLimit: OutputLimit, options: Engine.Options = .init(), resources: HostResources = .init()) throws -> PublicRenderOutcome {
            try renderOperation(options: options, resources: resources, outputLimit: outputLimit)
        }
        private func renderOperation(options: Engine.Options, resources: HostResources, outputLimit: OutputLimit?, presentationBudget: PresentationBudget? = nil) throws -> PublicRenderOutcome {
            let settings = try RenderNumberEncoding(options, outputLimit: outputLimit)
            let presentation = try presentationBudget?.packet()
            if presentation == nil, let detached = try renderDetached(settings: settings, resources: resources) {
                return detached
            }
            return try withHandle { handle in
                let response = resources.withCapturedNativeResources { host, takeError in
                    withHostFunctionResources(takeError) {
                        return settings.withPacket { packet in settings.optionsJSON.withUnsafeBytes { bytes in
                    if var value = presentation {
                        return withUnsafePointer(to: &value) { presentation in
                            pele_public_render_presentation_v11(handle, revision, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, packet, settings.outputKind, settings.outputBits, presentation, host)
                        }
                    }
                    return pele_public_render_operation_with_numbers_v9(handle, revision, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, packet, settings.outputKind, settings.outputBits, host)
                }}}}
                defer { clearHostFunctionErrors() }
                do {
                    let exchange = try Pele.binaryResult(response.handle)
                    let outcome = PublicRenderOutcome(revision: exchange.revision, graph: exchange.graph, result: try JSONDecoder().decode(PublicRenderResult.self, from: exchange.metadata))
                    revision = outcome.revision
                    if let original = originalHostFunctionError(outcome.result) { throw original }
                    if let error = response.error { throw error }
                    return outcome
                } catch { if let original = response.error { throw original }; throw error }
            }
        }
        public func render(options: Engine.Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            try renderedValue(renderOperation(options: options, resources: resources))
        }
        public func render(outputLimit: OutputLimit, options: Engine.Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            try renderedValue(renderOperation(outputLimit: outputLimit, options: options, resources: resources))
        }
        public func renderPresentation(outputLimit: OutputLimit? = nil, budget: PresentationBudget = .init(), options: Engine.Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            let limit = outputLimit ?? options.outputLimit.map { OutputLimit.number(Double($0)) } ?? .undefined
            return try renderedValue(renderOperation(options: options, resources: resources, outputLimit: limit, presentationBudget: budget))
        }
        @MainActor public func renderNative(nativeResources: NativeResources, outputLimit: OutputLimit? = nil,
            budget: PresentationBudget = .init(), options: Engine.Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            let (options, resources) = try nativeResources.resolved(options, resources: resources)
            let limit = outputLimit ?? options.outputLimit.map { OutputLimit.number(Double($0)) } ?? .undefined
            let settings = try RenderNumberEncoding(options, outputLimit: limit)
            var presentation = try budget.packet()
            return try withHandle { handle in
                let (response, imageError) = nativeResources.captured { native in resources.withCapturedNativeResources { host, takeError in
                    withHostFunctionResources(takeError) { withUnsafePointer(to: &presentation) { presentation in settings.withPacket { packet in settings.optionsJSON.withUnsafeBytes { bytes in
                        pele_public_render_native_v14(handle, revision, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count,
                            packet, settings.outputKind, settings.outputBits, presentation, host, native)
                    }}}
                }}}
                defer { clearHostFunctionErrors() }
                do {
                    let exchange = try Pele.binaryResult(response.handle)
                    let outcome = PublicRenderOutcome(revision: exchange.revision, graph: exchange.graph, result: try JSONDecoder().decode(PublicRenderResult.self, from: exchange.metadata))
                    revision = outcome.revision
                    if let original = originalHostFunctionError(outcome.result) { throw original }
                    if let error = response.error ?? imageError { throw error }
                    return try renderedValue(outcome)
                } catch { throw response.error ?? imageError ?? error }
            }
        }
        private func renderedValue(_ outcome: PublicRenderOutcome) throws -> FullRenderedDiagram {
            switch outcome.result {
            case .rendered(let rendered): return rendered
            case .thrown(.pele(let error)): throw error
            case .thrown(let error): throw PublicOperationFailure(value: error)
            }
        }
    }
}
extension Pele.Engine {
    /// Parse a supported mutable family. Unimplemented codecs report an explicit error.
    public func parseMutable(_ source: String, options: ParseOptions = .init()) throws -> Pele.MutableDocument {
        let source = Data(source.utf8), settings = try JSONEncoder().encode(options)
        var document: OpaquePointer?
        let result = source.withUnsafeBytes { input in settings.withUnsafeBytes { output in
            pele_public_parse_v4(input.bindMemory(to: UInt8.self).baseAddress, input.count,
                                 output.bindMemory(to: UInt8.self).baseAddress, output.count, &document)
        }}
        return try ownedPublicDocument(result, document: document)
    }
    /// Exact number-valued source threshold; overrides only options.limit.
    /// Config and integer maxEdges retain their existing contract. This accepts
    /// well-formed Swift strings; malformed UTF-16 source remains a separate gap.
    public func parseMutable(_ source: String, sourceLimit: Pele.SourceLimit, options: ParseOptions = .init()) throws -> Pele.MutableDocument {
        let source = Data(source.utf8), settings = try JSONEncoder().encode(options)
        let limit = sourceLimit.nativeValue
        var document: OpaquePointer?
        let result = source.withUnsafeBytes { input in settings.withUnsafeBytes { output in
            pele_public_parse_source_limit_v6(input.bindMemory(to: UInt8.self).baseAddress, input.count,
                                             output.bindMemory(to: UInt8.self).baseAddress, output.count,
                                             limit.kind, limit.bits, &document)
        }}
        return try ownedPublicDocument(result, document: document)
    }
    /// Override source and edge thresholds independently. Source is well-formed
    /// Swift text; native byte/work budgets remain independent of numeric limits.
    public func parseMutable(_ source: String, limits: Pele.ParseLimits, options: ParseOptions = .init()) throws -> Pele.MutableDocument {
        let source = Data(source.utf8), settings = try JSONEncoder().encode(options)
        let sourceLimit = limits.source?.nativeValue ?? (kind: UInt32(3), bits: UInt64(0))
        let edgeLimit = limits.maxEdges?.nativeValue ?? (kind: UInt32(3), bits: UInt64(0))
        var document: OpaquePointer?
        let result = source.withUnsafeBytes { input in settings.withUnsafeBytes { output in
            pele_public_parse_limits_v7(input.bindMemory(to: UInt8.self).baseAddress, input.count,
                                       output.bindMemory(to: UInt8.self).baseAddress, output.count,
                                       sourceLimit.kind, sourceLimit.bits, edgeLimit.kind, edgeLimit.bits, &document)
        }}
        return try ownedPublicDocument(result, document: document)
    }

}
