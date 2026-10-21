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
        /// Retains the supplied value without property reads or eager enumeration.
        /// The value store belongs to the returned document; ignored factories do
        /// not retain the argument. Callbacks may reenter that document.
        public func construct(firstArgument: PublicHostValue) throws -> MutableDocument {
            guard let handle else { throw PeleError(code: "semantic", message: "Public constructor is closed") }
            return try withExtendedLifetime(self) {
                struct Policy: Decodable { let ignoresArguments: Bool }
                let policy = try Pele.compatibilityResult(pele_public_constructor_policy_v17(handle), as: Policy.self)
                let store = PublicHostStore()
                var packet = PeleHostValueV17()
                packet.version = 17; packet.struct_size = UInt32(MemoryLayout<PeleHostValueV17>.size)
                if !policy.ignoresArguments {
                    if case .undefined = firstArgument { packet.tag = 0 }
                    else { packet.tag = 5; packet.payload = UInt64(try store.retain(firstArgument)) }
                }
                var document: OpaquePointer?
                let response = pele_public_constructor_construct_with_v17(handle, &packet, &document)
                let result = try ownedPublicDocument(response, document: document)
                result.hostValueStore = store
                result.usesHostValues = !policy.ignoresArguments
                return result
            }
        }
        public func constructPrepared(firstArgument: PublicHostValue) throws -> PreparedDocument {
            PreparedDocument(try construct(firstArgument: firstArgument))
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
