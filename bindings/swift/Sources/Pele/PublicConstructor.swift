import Foundation
import CPele

extension Pele {
    /// Captured native constructor. Confine to one owner/actor; not Sendable.
    /// Retains its builtin registration after registry closure or replacement.
    /// Creates low-level mutable state, not core parse provenance.
    public final class PublicConstructor {
        internal var handle: OpaquePointer?
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
        /// Allocate an unpublished receiver and ordered native constructor job.
        /// Host closures may capture the returned owner before calling run().
        public func beginConstruct(arguments: [PublicHostValue] = []) throws -> PublicConstruction {
            guard let handle else { throw PeleError(code: "semantic", message: "Public constructor is closed") }
            struct Policy: Decodable { let argumentCount: UInt32; let arity: UInt32; let argumentTransfers: [String] }
            let policy = try Pele.compatibilityResult(pele_public_constructor_policy_v18(handle), as: Policy.self)
            guard policy.argumentCount <= 2, policy.argumentTransfers.count == Int(policy.argumentCount),
                  policy.argumentTransfers.allSatisfy({ $0 == "retain" || $0 == "value" }) else {
                throw PeleError(code: "internal", message: "Invalid constructor argument policy")
            }
            let store = PublicHostStore()
            // Nested scopes keep both UTF16 buffers alive through the native call.
            // Extra arguments are never classified or retained.
            func withArgument<T>(_ index: Int, _ body: (UnsafePointer<PeleHostValueV17>) throws -> T) throws -> T {
                let argument: PublicHostValue = index < policy.argumentCount && index < arguments.count ? arguments[index] : .undefined
                if index < policy.argumentCount && policy.argumentTransfers[index] == "value" {
                    var retained: [UInt32] = []
                    return try argument.withCValue(store: store, temporary: &retained, body)
                }
                var packet = PeleHostValueV17(); packet.version = 17; packet.struct_size = UInt32(MemoryLayout<PeleHostValueV17>.size)
                if case .undefined = argument {} else { packet.tag = 5; packet.payload = UInt64(try store.retain(argument)) }
                return try withUnsafePointer(to: &packet, body)
            }
            var job: OpaquePointer?
            struct Created: Decodable { let revision: String }
            do {
                let metadata = try withExtendedLifetime(self) {
                    try withArgument(0) { first in
                        try withArgument(1) { second in
                            try Pele.compatibilityResult(pele_public_constructor_begin_v18(handle, first, second, &job), as: Created.self)
                        }
                    }
                }
                guard let job, let revision = UInt64(metadata.revision) else { throw PeleError(code: "internal", message: "Invalid construction job") }
                return PublicConstruction(handle: job, type: type, revision: revision, store: store)
            } catch { if let job { pele_public_construction_free_v18(job) }; throw error }
        }
        public func construct(arguments: [PublicHostValue]) throws -> MutableDocument { try beginConstruct(arguments: arguments).run() }
        public func constructPrepared(arguments: [PublicHostValue]) throws -> PreparedDocument { PreparedDocument(try construct(arguments: arguments)) }
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
