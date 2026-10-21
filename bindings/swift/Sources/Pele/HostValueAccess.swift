import Foundation
import CPele

/// The native owner retains this box, which does not retain the Swift document.
/// User values remain in the document's existing actor-confined host ledger.
final class HostValueAccessBox {
    weak var document: Pele.MutableDocument?
    var errors: [UInt32: any Error] = [:]
    let exceptionToken: UInt32
    init(_ document: Pele.MutableDocument, exceptionToken: UInt32) { self.document = document; self.exceptionToken = exceptionToken }
}
private final class HostValueReply {
    let units: UnsafeMutablePointer<UInt16>?
    let count: Int
    init(_ text: Pele.JSString?) {
        count = text?.codeUnits.count ?? 0
        if let text, !text.codeUnits.isEmpty {
            let storage = UnsafeMutablePointer<UInt16>.allocate(capacity: count)
            text.codeUnits.withUnsafeBufferPointer { source in storage.initialize(from: source.baseAddress!, count: count) }
            units = storage
        } else { units = nil }
    }
    deinit { units?.deallocate() }
}
private func accessRetain(_ context: UnsafeMutableRawPointer?) {
    if let context { _ = Unmanaged<HostValueAccessBox>.fromOpaque(context).retain() }
}
private func accessRelease(_ context: UnsafeMutableRawPointer?) {
    if let context { Unmanaged<HostValueAccessBox>.fromOpaque(context).release() }
}
private func replyRelease(_ context: UnsafeMutableRawPointer?) {
    if let context { Unmanaged<HostValueReply>.fromOpaque(context).release() }
}
private func accessInvoke(_ context: UnsafeMutableRawPointer?, _ input: UnsafePointer<PeleHostRequestV20>?, _ output: UnsafeMutablePointer<PeleHostReplyV20>?) {
    guard let context, let input, let output else { return }
    let box = Unmanaged<HostValueAccessBox>.fromOpaque(context).takeUnretainedValue()
    guard let document = box.document else { return }
    func text(_ pointer: UnsafePointer<UInt16>?, _ count: Int) throws -> Pele.JSString {
        guard count >= 0, count <= 16_000_000, count == 0 || pointer != nil else { throw invalidHost("Invalid host UTF16 lease") }
        return .init(codeUnits: Array(UnsafeBufferPointer(start: pointer, count: count)))
    }
    func write(_ value: Pele.PublicHostValue, retained: Bool = false) throws {
        var packet = PeleHostValueV17()
        packet.version = 17; packet.struct_size = UInt32(MemoryLayout<PeleHostValueV17>.size)
        var string: Pele.JSString?
        if retained {
            packet.tag = 5; packet.payload = UInt64(try document.hostValueStore.retain(value, reuseIdentity: false))
        } else {
            switch value {
            case .undefined: packet.tag = 0
            case .null: packet.tag = 1
            case .bool(let value): packet.tag = 2; packet.payload = value ? 1 : 0
            case .number(let value): packet.tag = 3; packet.payload = value.bitPattern
            case .string(let value):
                guard value.codeUnits.count <= 16_000_000 else { throw PeleError(code: "limit", message: "Opaque host reply exceeds UTF16 budget") }
                packet.tag = 4; string = value
            case .host: packet.tag = 5; packet.payload = UInt64(try document.hostValueStore.retain(value))
            case .graphReference(let value): packet.tag = 6; packet.payload = UInt64(value)
            case .builtinFunction(let value): packet.tag = 7; packet.payload = UInt64(value)
            }
        }
        let storage = HostValueReply(string)
        packet.units = storage.units.map { UnsafePointer($0) }; packet.unit_count = storage.count
        output.pointee.value = packet
        output.pointee.owner = Unmanaged.passRetained(storage).toOpaque()
        output.pointee.release = replyRelease
        output.pointee.status = 1
    }
    do {
        let request = input.pointee
        guard request.count >= 0, request.count <= 1_000_002, request.count == 0 || request.values != nil else { throw invalidHost("Invalid host operand lease") }
        let packets = UnsafeBufferPointer(start: request.values, count: request.count)
        // C12 function bindings occupy the same globally unique token space,
        // but only this explicit owner ledger selects opaque dispatch.
        if request.operation == 1, let function = packets.first, function.tag == 5,
           let id = UInt32(exactly: function.payload), !document.hostValueStore.contains(id) {
            if document.hostFunctions[id] != nil { return }
            throw invalidHost("Unknown owner host identity")
        }
        func value(_ packet: PeleHostValueV17) throws -> Pele.PublicHostValue {
            guard packet.version == 17, packet.struct_size == MemoryLayout<PeleHostValueV17>.size, packet.reserved == 0 else { throw invalidHost("Invalid host value packet") }
            switch packet.tag {
            case 0: return .undefined
            case 1: return .null
            case 2: guard packet.payload <= 1 else { throw invalidHost("Invalid host Boolean") }; return .bool(packet.payload != 0)
            case 3: return .number(Double(bitPattern: packet.payload))
            case 4: return .string(try text(packet.units, packet.unit_count))
            case 5: guard let id = UInt32(exactly: packet.payload) else { throw invalidHost("Invalid host identity") }; return try document.hostValueStore.value(id)
            case 6: guard let id = UInt32(exactly: packet.payload) else { throw invalidHost("Invalid graph coordinate") }; return .graphReference(id)
            case 7: guard let id = UInt32(exactly: packet.payload) else { throw invalidHost("Invalid builtin identity") }; return .builtinFunction(id)
            default: throw invalidHost("Invalid host value tag")
            }
        }
        let kinds = ["get", "call", "truthy", "toString", "greaterEqual", "strictEqual", "toNumber"]
        guard Int(request.operation) < kinds.count, request.result <= 2 else { throw invalidHost("Invalid host operation") }
        let key = try text(request.key, request.key_count)
        let message = try text(request.error, request.error_count)
        guard let keyText = key.stringIfWellFormed, let errorText = message.stringIfWellFormed else { throw invalidHost("Invalid native operation text") }
        let result = try document.performHostValues(kind: kinds[Int(request.operation)], values: packets.map(value),
            key: keyText, notCallableError: errorText)
        if request.operation == 1, request.result == 2 { try write(.undefined) }
        else { try write(result, retained: request.operation == 1 && request.result == 1) }
    } catch let error as HostTypeError {
        do { try write(.string(.init(error.message))); output.pointee.status = 3 }
        catch { box.errors[box.exceptionToken] = error; output.pointee.status = 2; output.pointee.exception = box.exceptionToken }
    } catch {
        box.errors[box.exceptionToken] = error; output.pointee.status = 2; output.pointee.exception = box.exceptionToken
    }
}
extension Pele.MutableDocument {
    func ensureHostValueAccess(_ handle: OpaquePointer) throws {
        guard hostValueAccess == nil else { return }
        let token = try Pele.compatibilityResult(pele_host_identity_v17(), as: UInt32.self)
        let box = HostValueAccessBox(self, exceptionToken: token)
        var table = PeleHostAccessV20()
        table.version = 20; table.struct_size = UInt32(MemoryLayout<PeleHostAccessV20>.size)
        table.context = Unmanaged.passUnretained(box).toOpaque()
        table.retain = accessRetain; table.release = accessRelease; table.invoke = accessInvoke
        struct Installed: Decodable { let installed: Bool }
        let result = try withExtendedLifetime(box) {
            try Pele.compatibilityResult(pele_public_set_host_access_v20(handle, &table), as: Installed.self)
        }
        guard result.installed else { throw invalidHost("Host access registration failed") }
        hostValueAccess = box
    }
}
