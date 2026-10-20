import Foundation
import CPele

extension Pele {
    /// Captured native constructor. Confine to one owner/actor; not Sendable.
    /// Retains its builtin registration after registry closure or replacement.
    /// Creates low-level mutable state, not core parse provenance.
    public final class PublicConstructor {
        private var handle: OpaquePointer?
        public let type: String
        public let prototype: UInt32
        fileprivate init(handle: OpaquePointer, type: String, prototype: UInt32) {
            self.handle = handle; self.type = type; self.prototype = prototype
        }
        deinit { if let handle { pele_public_constructor_free_v16(handle) } }
        public func close() {
            if let handle { self.handle = nil; pele_public_constructor_free_v16(handle) }
        }
        /// Native defaults; no source parsing or caller graph seeding.
        public func construct() throws -> MutableDocument {
            guard let handle else { throw PeleError(code: "semantic", message: "Public constructor is closed") }
            return try withExtendedLifetime(self) {
                var document: OpaquePointer?
                let result = pele_public_constructor_construct_v16(handle, &document)
                return try ownedPublicDocument(result, document: document)
            }
        }
        /// A fresh independent model using the named low-level render contract.
        public func constructPrepared() throws -> PreparedDocument {
            PreparedDocument(try construct())
        }
    }
}
extension Pele.Registry {
    /// Nil means this registration/prototype has no native constructor port.
    /// Prototype IDs come from the registration's public model schema.
    public func captureConstructor(for type: String, prototype: UInt32) throws -> Pele.PublicConstructor? {
        struct Selection: Decodable { let selected: Bool }
        let bytes = Data(type.utf8)
        return try withHandle { registry in
            var constructor: OpaquePointer?
            let result = bytes.withUnsafeBytes { input in
                pele_registry_capture_constructor_v16(registry,
                    input.bindMemory(to: UInt8.self).baseAddress, input.count, prototype, &constructor)
            }
            do {
                let selection = try Pele.compatibilityResult(result, as: Selection.self)
                guard selection.selected else {
                    if let constructor { pele_public_constructor_free_v16(constructor) }
                    return nil
                }
                guard let constructor else { throw PeleError(code: "internal", message: "Missing public constructor") }
                return Pele.PublicConstructor(handle: constructor, type: type, prototype: prototype)
            } catch {
                if let constructor { pele_public_constructor_free_v16(constructor) }
                throw error
            }
        }
    }
}
