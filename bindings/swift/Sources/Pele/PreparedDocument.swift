import Foundation
import CPele

extension Pele {
    /// Low-level per-diagram source context. Source is already encoded/prepared;
    /// no newline, comment removal, title/config merge or default limit is added.
    public struct PreparedParseOptions: Sendable {
        public var config: [String: ConfigValue] = [:]
        public var title: String?
        public var lineOffset: UInt64 = 0
        /// Raw undefined disables the semantic edge cap. Native work bounds remain.
        public var maxEdges: EdgeLimit = .undefined
        public init() {}
    }
    /// A separate low-level owner, deliberately not accepted by Engine.render.
    /// Graphs/methods retain the same native schema and identity as other documents.
    public final class PreparedDocument {
        private let document: MutableDocument
        public var type: String { document.type }
        public var revision: UInt64 { document.revision }
        public var isConstructorCreated: Bool { document.isConstructorCreated }
        internal init(_ document: MutableDocument) { self.document = document }
        public func close() throws { try document.close() }
        public func snapshot() throws -> PublicSnapshot { try document.snapshot() }
        @discardableResult public func apply(_ graph: PublicGraph, expectedRevision: UInt64) throws -> UInt64 {
            try document.apply(graph, expectedRevision: expectedRevision)
        }
        public func schema() throws -> PublicModelSchema { try document.schema() }
        public func bindHostFunction(_ function: HostFunction) throws -> PublicGraph.Atom { try document.bindHostFunction(function) }
        public func invoke(method: UInt32, receiver: PublicGraph.Atom, arguments: [PublicGraph.Atom] = [], resources: HostResources = .init()) throws -> PublicInvocationOutcome {
            try document.invoke(method: method, receiver: receiver, arguments: arguments, resources: resources)
        }
        /// Direct Diagram.render: supplied config/options replace core defaults.
        /// The native result additionally identifies its retained diagram type.
        public func renderOperation(config: [String: ConfigValue] = [:], options: Engine.Options = .init(), resources: HostResources = .init()) throws -> PublicRenderOutcome {
            let config = try JSONEncoder().encode(config)
            var raw = options; raw.config = [:]; raw.limit = nil; raw.maxEdges = nil
            let settings = try RenderNumberEncoding(raw, outputLimit: .undefined)
            if let detached = try document.renderDetached(settings: settings, resources: resources, rawConfig: config) {
                return detached
            }
            return try document.withHandle { handle in
                let response = resources.withCapturedNativeResources { host, takeError in
                    document.withHostFunctionResources(takeError) {
                    config.withUnsafeBytes { config in settings.withPacket { numbers in settings.optionsJSON.withUnsafeBytes { bytes in
                        pele_render_prepared_v13(handle, document.revision,
                            config.bindMemory(to: UInt8.self).baseAddress, config.count,
                            bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, numbers, host)
                    }}}
                    }
                }
                defer { document.clearHostFunctionErrors() }
                do {
                    let exchange = try Pele.binaryResult(response.handle)
                    let outcome = PublicRenderOutcome(revision: exchange.revision, graph: exchange.graph,
                        result: try JSONDecoder().decode(PublicRenderResult.self, from: exchange.metadata))
                    document.acceptPreparedRevision(outcome.revision)
                    if let error = response.error { throw error }
                    if let original = document.originalHostFunctionError(outcome.result) { throw original }
                    return outcome
                } catch { if let original = response.error { throw original }; throw error }
            }
        }
        public func render(config: [String: ConfigValue] = [:], options: Engine.Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            switch try renderOperation(config: config, options: options, resources: resources).result {
            case .rendered(let drawing): return drawing
            case .thrown(.pele(let error)): throw error
            case .thrown(let error): throw PublicOperationFailure(value: error)
            }
        }
    }
}
extension Pele.Engine {
    /// Equivalent of a named Diagram.parse(context), independently of registration.
    /// Caller supplies verbatim prepared text and exact nonnegative line offset.
    public func parsePrepared(_ type: String, source: String, options: Pele.PreparedParseOptions = .init()) throws -> Pele.PreparedDocument {
        let type = Data(type.utf8), source = Data(source.utf8), config = try JSONEncoder().encode(options.config)
        let title = Data((options.title ?? "").utf8), edge = options.maxEdges.nativeValue
        var packet = PelePreparedOptionsV13()
        packet.version = 13; packet.struct_size = UInt32(MemoryLayout<PelePreparedOptionsV13>.size)
        packet.line_offset = options.lineOffset; packet.max_edges_kind = edge.kind
        packet.max_edges_bits = edge.bits; packet.title_present = options.title == nil ? 0 : 1
        var document: OpaquePointer?
        let result = withUnsafePointer(to: &packet) { packet in
            type.withUnsafeBytes { type in source.withUnsafeBytes { source in config.withUnsafeBytes { config in title.withUnsafeBytes { title in
                pele_parse_prepared_v13(type.bindMemory(to: UInt8.self).baseAddress, type.count,
                    source.bindMemory(to: UInt8.self).baseAddress, source.count,
                    config.bindMemory(to: UInt8.self).baseAddress, config.count,
                    title.bindMemory(to: UInt8.self).baseAddress, title.count, packet, &document)
            }}}}
        }
        return Pele.PreparedDocument(try ownedPublicDocument(result, document: document))
    }
}

extension Pele.Engine {
    /// Create directly from caller-owned model data, without parsing source.
    public func createPrepared(_ type: String, graph: Pele.PublicGraph, config: [String: ConfigValue] = [:]) throws -> Pele.PreparedDocument {
        let type = Data(type.utf8), graph = try graph.binaryBytes(revision: 0), config = try JSONEncoder().encode(config)
        var document: OpaquePointer?
        let result = type.withUnsafeBytes { type in graph.withUnsafeBytes { graph in config.withUnsafeBytes { config in
            pele_create_prepared_v13(type.bindMemory(to: UInt8.self).baseAddress, type.count,
                graph.bindMemory(to: UInt8.self).baseAddress, graph.count,
                config.bindMemory(to: UInt8.self).baseAddress, config.count, &document)
        }}}
        return Pele.PreparedDocument(try ownedPublicDocument(result, document: document))
    }
    /// Inspect a named model schema without constructing or parsing a document.
    public func preparedSchema(for type: String) throws -> Pele.PublicModelSchema {
        try Data(type.utf8).withUnsafeBytes { type in
            try Pele.compatibilityResult(pele_prepared_schema_v13(type.bindMemory(to: UInt8.self).baseAddress, type.count), as: Pele.PublicModelSchema.self)
        }
    }
}
