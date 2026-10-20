import Foundation
import CPele

public struct DiagramLink: Codable, Sendable {
    public let id: String; public let href: String; public let `internal`: Bool
    public init(id: String, href: String, internal isInternal: Bool = false) {
        self.id = id; self.href = href; self.internal = isInternal
    }
}
public struct FullRenderedDiagram: Codable, Sendable {
    public let type: String
    public let svg: String
    public let width: Double
    public let height: Double
    public let links: [DiagramLink]
    public let presentation: Pele.PresentationOutcome?
    private enum CodingKeys: String, CodingKey { case type, svg, width, height, links, presentation; case numbers = "$numbers" }
    public init(from decoder: any Decoder) throws {
        let fields = try decoder.container(keyedBy: CodingKeys.self)
        type = try fields.decode(String.self, forKey: .type)
        svg = try fields.decode(String.self, forKey: .svg)
        links = try fields.decode([DiagramLink].self, forKey: .links)
        presentation = try fields.decodeIfPresent(Pele.PresentationOutcome.self, forKey: .presentation)
        let numbers = try fields.decodeIfPresent([String: String].self, forKey: .numbers) ?? [:]
        func number(_ key: CodingKeys) throws -> Double {
            if let hex = numbers[key.rawValue] {
                guard hex.count == 16, let bits = UInt64(hex, radix: 16) else {
                    throw DecodingError.dataCorruptedError(forKey: key, in: fields, debugDescription: "Invalid IEEE dimension bits")
                }
                return Double(bitPattern: bits)
            }
            return try fields.decode(Double.self, forKey: key)
        }
        width = try number(.width); height = try number(.height)
    }
    public func encode(to encoder: any Encoder) throws {
        var fields = encoder.container(keyedBy: CodingKeys.self)
        try fields.encode(type, forKey: .type); try fields.encode(svg, forKey: .svg); try fields.encode(links, forKey: .links)
        try fields.encodeIfPresent(presentation, forKey: .presentation)
        var numbers: [String: String] = [:]
        for (key, value) in [(CodingKeys.width, width), (.height, height)] {
            if value.isFinite { try fields.encode(value, forKey: key) } else { try fields.encodeNil(forKey: key) }
            if !value.isFinite || (value == 0 && value.sign == .minus) { numbers[key.rawValue] = String(format: "%016llx", value.bitPattern) }
        }
        if !numbers.isEmpty { try fields.encode(numbers, forKey: .numbers) }
    }
}
private struct DocumentProjectionMetadata: Decodable { let type: String; let projectionBytes: Int }

extension Pele {
    /// Owned immutable parse result. Rust registration/config/model survive until deinit.
    public final class Document: @unchecked Sendable {
        internal let handle: OpaquePointer
        public let type: String
        public let model: ConfigValue
        fileprivate init(handle: OpaquePointer, type: String, model: ConfigValue) {
            self.handle = handle; self.type = type; self.model = model
        }
        deinit { pele_result_free(handle) }
        /// Read-only operation on the exact retained Rust model.
        public func query(_ operation: String, arguments: ConfigValue = .null) throws -> ConfigValue {
            let name = Data(operation.utf8), arguments = try JSONEncoder().encode(arguments)
            return try name.withUnsafeBytes { name in try arguments.withUnsafeBytes { arguments in
                try Pele.compatibilityResult(pele_document_query_v2(handle,
                    name.bindMemory(to: UInt8.self).baseAddress, name.count,
                    arguments.bindMemory(to: UInt8.self).baseAddress, arguments.count), as: ConfigValue.self)
            }}
        }
        public func decodeModel<T: Decodable>(as: T.Type) throws -> T {
            try JSONDecoder().decode(T.self, from: JSONEncoder().encode(model))
        }
    }
    /// Source-built compatibility entry; capabilities identify the implemented built-ins.
    public struct NativeTheme: Codable, Sendable {
        public var background = "#ffffff", foreground = "#1f1f1f", muted = "#6e6e6e", line = "#1f1f1f"
        public var surface = "#f3f3f3", surfaceAlt = "#e6e6e6", border = "#8a8a8a", accent = "#5b7bd5"
        public var series = ["#4c78a8", "#f58518", "#54a24b", "#e45756", "#72b7b2", "#eeca3b", "#b279a2", "#9d755d"]
        public init() {}
    }
    public struct Engine: Sendable {
        public struct ParseOptions: Codable, Sendable {
            public var limit = 50_000
            public var maxEdges = 5_000
            public var config: [String: ConfigValue] = [:]
            public init() {}
        }
        public struct Options: Codable, Sendable {
            public var limit: Int?
            public var maxEdges: Int?
            public var config: [String: ConfigValue] = [:]
            public var fontFamily: String?
            public var fontFamilyMono: String?
            public var fontSize: Double?
            public var idPrefix: String?
            public var responsive: Bool?
            public var maxWidth: Double?
            public var autoDirection: Bool?
            public var directionBreakpoint: Double?
            public var padding: Double?
            public var outputLimit: Int?
            public var linkSchemes: [String]?
            public var imageSchemes: [String]?
            public var links: Bool?
            public var images: Bool?
            public var linkRel: String?
            public var now: Double?
            public var nativeTheme: NativeTheme?
            public init() {}
        }
        public init() {}
        public func capabilities() throws -> [DiagramCapability] {
            try Pele.compatibilityResult(pele_builtin_capabilities(), as: [DiagramCapability].self)
        }
        public func parse(_ source: String, options: ParseOptions = .init()) throws -> Document {
            let bytes = Data(source.utf8), settings = try JSONEncoder().encode(options)
            let projection = EagerProjection()
            let handle = withExtendedLifetime(projection) { bytes.withUnsafeBytes { input in settings.withUnsafeBytes { output in
                pele_parse_eager_v1(input.bindMemory(to: UInt8.self).baseAddress, input.count,
                    output.bindMemory(to: UInt8.self).baseAddress, output.count,
                    Unmanaged.passUnretained(projection).toOpaque(), receiveEagerProjection)
            }}}
            guard let handle else { throw PeleError(code: "internal", message: "Native document allocation failed") }
            do {
                // A Swift conversion error aborts the callback and is rethrown
                // here; it never unwinds across the C boundary.
                if let error = projection.error { throw error }
                let metadata = try Pele.compatibilityResult(handle, as: DocumentProjectionMetadata.self, release: false)
                guard metadata.projectionBytes >= 0, metadata.projectionBytes <= 64 * 1024 * 1024 else {
                    throw PeleError(code: "internal", message: "Invalid native compatibility result")
                }
                return Document(handle: handle, type: metadata.type, model: try projection.finish())
            } catch { pele_result_free(handle); throw error }
        }
        public func render(_ source: String, options: Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            try render(source, options: options, resources: resources, outputLimit: nil)
        }
        public func render(_ source: String, outputLimit: OutputLimit, options: Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            try render(source, options: options, resources: resources, outputLimit: outputLimit)
        }
        private func render(_ source: String, options: Options, resources: HostResources, outputLimit: OutputLimit?) throws -> FullRenderedDiagram {
            let bytes = Data(source.utf8), settings = try RenderNumberEncoding(options, outputLimit: outputLimit)
            let handle = try resources.withNativeResources { host in settings.withPacket { packet in bytes.withUnsafeBytes { input in settings.optionsJSON.withUnsafeBytes { output in
                pele_render_with_numbers_v9(input.bindMemory(to: UInt8.self).baseAddress, input.count,
                    output.bindMemory(to: UInt8.self).baseAddress, output.count, packet, settings.outputKind, settings.outputBits, host)
            }}}}
            return try Pele.compatibilityResult(handle, as: FullRenderedDiagram.self)
        }
        public func render(_ document: Document, options: Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            try render(document, options: options, resources: resources, outputLimit: nil)
        }
        public func render(_ document: Document, outputLimit: OutputLimit, options: Options = .init(), resources: HostResources = .init()) throws -> FullRenderedDiagram {
            try render(document, options: options, resources: resources, outputLimit: outputLimit)
        }
        private func render(_ document: Document, options: Options, resources: HostResources, outputLimit: OutputLimit?) throws -> FullRenderedDiagram {
            let settings = try RenderNumberEncoding(options, outputLimit: outputLimit)
            let handle = try resources.withNativeResources { host in withExtendedLifetime(document) { settings.withPacket { packet in settings.optionsJSON.withUnsafeBytes { output in
                pele_render_document_with_numbers_v9(document.handle, output.bindMemory(to: UInt8.self).baseAddress, output.count,
                    packet, settings.outputKind, settings.outputBits, host)
            }}}}
            return try Pele.compatibilityResult(handle, as: FullRenderedDiagram.self)
        }
    }
}
