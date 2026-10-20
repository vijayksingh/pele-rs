import Foundation
import CPele

/// One parse-local eager builder. No borrowed native storage survives a callback.
/// Frames are reference-owned so appending does not copy partially built trees.
internal final class EagerProjection {
    private final class Frame {
        let object: Bool
        let expected: Int
        var received = 0
        var array: [ConfigValue] = []
        var dictionary: [String: ConfigValue] = [:]
        var key: String?
        init(object: Bool, count: Int) {
            self.object = object; expected = count
            if object { dictionary.reserveCapacity(count) }
            else { array.reserveCapacity(count) }
        }
    }
    private var frames: [Frame] = []
    private var root: ConfigValue?
    private(set) var error: (any Error)?

    private func invalid() -> PeleError {
        PeleError(code: "internal", message: "Invalid native compatibility result")
    }
    private func string(_ event: PeleProjectionEventV1) throws -> String {
        guard event.count >= 0, event.count <= 64 * 1024 * 1024,
              event.count == 0 || event.text != nil else { throw invalid() }
        let bytes = UnsafeBufferPointer(start: event.text, count: event.count)
        // Foundation's encoding initializer consumes a leading UTF-8 BOM.
        // These are model string bytes, so U+FEFF is data, not a signature.
        let value = String(decoding: bytes, as: UTF8.self)
        // Preserve strict validation on macOS 14 too: the newer failable
        // standard-library UTF-8 constructor requires a newer deployment target.
        guard value.utf8.elementsEqual(bytes) else { throw invalid() }
        return value
    }
    private func append(_ value: ConfigValue) throws {
        guard let frame = frames.last else {
            guard root == nil else { throw invalid() }
            root = value; return
        }
        guard frame.received < frame.expected else { throw invalid() }
        if frame.object {
            guard let key = frame.key else { throw invalid() }
            // Foundation JSONDecoder preserves the first canonically equivalent
            // key and value. Native normalization already resolved exact JSON
            // duplicates before emitting these JS-ordered object entries.
            if frame.dictionary[key] == nil { frame.dictionary[key] = value }
            frame.key = nil
        } else { frame.array.append(value) }
        frame.received += 1
    }
    private func consume(_ event: PeleProjectionEventV1) throws {
        switch event.tag {
        case 0: try append(.null)
        case 1:
            guard event.count == 0 || event.count == 1 else { throw invalid() }
            try append(.bool(event.count != 0))
        case 2:
            guard event.number.isFinite else { throw invalid() }
            try append(.number(event.number))
        case 3: try append(.string(string(event)))
        case 4, 6:
            // The model already passed the native depth/byte gate. These checks
            // also keep malformed callback streams from allocating unboundedly.
            guard frames.count < 128, event.count >= 0,
                  event.count <= 16 * 1024 * 1024 else { throw invalid() }
            frames.append(Frame(object: event.tag == 6, count: event.count))
        case 5, 8:
            guard let frame = frames.last, frame.object == (event.tag == 8),
                  frame.key == nil, frame.received == frame.expected else { throw invalid() }
            frames.removeLast()
            try append(frame.object ? .object(frame.dictionary) : .array(frame.array))
        case 7:
            guard let frame = frames.last, frame.object, frame.key == nil,
                  frame.received < frame.expected else { throw invalid() }
            frame.key = try string(event)
        default: throw invalid()
        }
    }
    func receive(_ events: UnsafePointer<PeleProjectionEventV1>?, count: Int) -> UInt32 {
        guard error == nil else { return 1 }
        do {
            guard let events, count > 0, count <= 128 else { throw invalid() }
            for event in UnsafeBufferPointer(start: events, count: count) { try consume(event) }
            return 0
        } catch { self.error = error; return 1 }
    }
    func finish() throws -> ConfigValue {
        if let error { throw error }
        guard frames.isEmpty, let root else { throw invalid() }
        return root
    }
}

internal func receiveEagerProjection(_ context: UnsafeMutableRawPointer?,
    _ events: UnsafePointer<PeleProjectionEventV1>?, _ count: Int) -> UInt32 {
    guard let context else { return 1 }
    return Unmanaged<EagerProjection>.fromOpaque(context).takeUnretainedValue().receive(events, count: count)
}
