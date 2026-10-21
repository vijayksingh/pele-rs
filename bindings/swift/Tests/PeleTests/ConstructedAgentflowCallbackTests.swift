import Foundation
import Pele
import XCTest

private final class ConstructedAgentflowError: Error {}
private final class ConstructedAgentflowHost: Pele.PublicHostCallable {
  var get: (Pele.JSString) throws -> Pele.PublicHostValue = { _ in .undefined }
  var call: (Pele.PublicHostValue, [Pele.PublicHostValue]) throws -> Pele.PublicHostValue = {
    _, _ in .undefined
  }
  func publicGet(_ key: Pele.JSString) throws -> Pele.PublicHostValue { try get(key) }
  func publicCall(receiver: Pele.PublicHostValue, arguments: [Pele.PublicHostValue]) throws
    -> Pele.PublicHostValue
  { try call(receiver, arguments) }
}
@MainActor private final class ConstructedAgentflowState {
  var trace: [[Any]] = []
  var fired = false
  var nested: FullRenderedDiagram?
}
@MainActor final class ConstructedAgentflowCallbackTests: XCTestCase {
  private func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
    guard case .ref(let id) = value else {
      throw PeleError(code: "test", message: "Expected graph reference")
    }
    return id
  }
  private func json(_ value: Any) throws -> ConfigValue {
    try JSONDecoder().decode(
      ConfigValue.self,
      from: JSONSerialization.data(withJSONObject: value, options: [.fragmentsAllowed]))
  }
  private func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
  private func input(_ value: Any, _ graph: inout Pele.PublicGraph) throws -> Pele.PublicGraph.Atom
  {
    if value is NSNull { return .null }
    if let text = value as? String { return .string(try graph.intern(text)) }
    if let number = value as? NSNumber { return .number(number.doubleValue) }
    let id = UInt32(graph.nodes.count)
    graph.nodes.append(.object(properties: []))
    if let values = value as? [Any] {
      graph.nodes[Int(id)] = .array(length: UInt32(values.count), properties: [])
      for (index, value) in values.enumerated() {
        let atom = try input(value, &graph)
        try graph.setArrayElement(UInt32(index), value: atom, on: id)
      }
    } else if let object = value as? [String: Any] {
      for key in object.keys.sorted() {
        let atom = try input(object[key]!, &graph)
        try graph.setField(key, value: atom, on: id)
      }
    } else {
      throw PeleError(code: "test", message: "Unsupported fixture argument")
    }
    return .ref(id)
  }
  private func invoke(_ document: Pele.PreparedDocument, _ method: UInt32, _ values: [Any]) throws {
    var graph = try document.snapshot().graph
    let args = try values.map { try input($0, &graph) }
    try document.apply(graph, expectedRevision: document.revision)
    let result = try document.invoke(method: method, receiver: graph.root, arguments: args)
    if case .thrown(let error) = result.result {
      throw PeleError(code: "test", message: "Unexpected helper failure: \(error)")
    }
  }
  private func node(_ graph: Pele.PublicGraph, _ key: String) throws -> UInt32 {
    let map = try ref(graph.field("nodes"))
    guard case .map(let entries, _) = graph.nodes[Int(map)] else {
      throw ConstructedAgentflowError()
    }
    for entry in entries {
      if case .string(let id) = entry.key, graph.strings[Int(id)].stringIfWellFormed == key {
        return try ref(entry.value)
      }
    }
    throw PeleError(code: "test", message: "Missing node")
  }
  private func text(
    _ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on id: UInt32? = nil
  ) throws {
    let value = try graph.intern(value)
    try graph.setField(key, value: .string(value), on: id)
  }
  private func observed(_ graph: Pele.PublicGraph, oldStyles: UInt32) throws -> ConfigValue {
    func value(_ atom: Pele.PublicGraph.Atom) throws -> Any {
      switch atom {
      case .undefined, .null: return NSNull()
      case .bool(let v): return v
      case .number(let v): return v.isFinite ? v as Any : NSNull()
      case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
      case .ref(let id):
        switch graph.nodes[Int(id)] {
        case .array(let count, _):
          return try (0..<count).map { try value(graph.field(String($0), on: id) ?? .undefined) }
        case .map(let entries, _): return try entries.map { try [value($0.key), value($0.value)] }
        default:
          var object: [String: Any] = [:]
          for p in graph.nodes[Int(id)].properties {
            if case .undefined = p.value { continue }
            object[try XCTUnwrap(graph.strings[Int(p.key)].stringIfWellFormed)] = try value(p.value)
          }
          return object
        }
      default: throw PeleError(code: "test", message: "Unexpected observed callable")
      }
    }
    let map = try ref(graph.field("nodes"))
    guard case .map(let entries, _) = graph.nodes[Int(map)] else {
      throw ConstructedAgentflowError()
    }
    let nodes: [[Any]] = try entries.map { entry in
      let id = try ref(entry.value)
      return [
        try value(entry.key),
        [
          "text": try value(graph.field("text", on: id) ?? .undefined),
          "styles": try value(graph.field("styles", on: id) ?? .undefined),
        ],
      ]
    }
    return try json([
      "title": value(graph.field("title") ?? .undefined), "nodes": nodes,
      "diagnostics": value(graph.field("diagnostics") ?? .undefined),
      "reported": value(graph.field("reported") ?? .undefined), "oldStyles": value(.ref(oldStyles)),
    ])
  }
  private func compare(_ drawing: FullRenderedDiagram, _ expected: [String: Any], _ id: String)
    throws
  {
    XCTAssertEqual(expected["ok"] as? Bool, true, id)
    XCTAssertEqual(drawing.svg, expected["svg"] as? String, id)
    XCTAssertEqual(bits(drawing.width), expected["width"] as? String, id)
    XCTAssertEqual(bits(drawing.height), expected["height"] as? String, id)
    let links = try JSONDecoder().decode(
      ConfigValue.self, from: JSONEncoder().encode(drawing.links))
    XCTAssertEqual(links, try json(XCTUnwrap(expected["links"])), id)
  }
  func
    testConstructedDiagnosticsRetainOriginalStyleHandlesAndSuccessfulNestedCacheAcross18OracleCases()
    throws
  {
    var path = URL(fileURLWithPath: #filePath)
    for _ in 0..<5 { path.deleteLastPathComponent() }
    let fixture = try XCTUnwrap(
      JSONSerialization.jsonObject(
        with: Data(
          contentsOf: path.appendingPathComponent(
            "fixtures/public-agentflow-constructed-live.json"))) as? [String: Any])
    XCTAssertEqual(
      fixture["oracleSHA256"] as? String,
      "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
    let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
    XCTAssertEqual(rows.count, 18)
    for row in rows {
      let trigger = try XCTUnwrap(row["trigger"] as? String)
      let mode = try XCTUnwrap(row["mode"] as? String)
      let id = trigger + "/" + mode
      let state = ConstructedAgentflowState()
      let original = ConstructedAgentflowError()
      let registry = try Pele.Registry.allBuiltins()
      let options = ConstructedAgentflowHost()
      let log = ConstructedAgentflowHost()
      options.get = { $0 == .init("log") ? .host(log) : .undefined }
      let factory = try XCTUnwrap(registry.captureConstructor(for: "agentflow", prototype: 1))
      let document = try factory.constructPrepared(firstArgument: .host(options))
      factory.close()
      try registry.close()
      defer { try? document.close() }
      var graph = try document.snapshot().graph
      try text(&graph, "title", "Initial")
      try document.apply(graph, expectedRevision: document.revision)
      try invoke(
        document, 3,
        ["A", ["text": "Alpha", "type": "text"], trigger == "shape" ? "circle" : "task"])
      try invoke(document, 3, ["B", ["text": "Beta", "type": "text"], "task"])
      try invoke(
        document, 4, ["A", "B", ["type": "arrow_point", "stroke": "normal", "length": 1]])
      if trigger == "containment" {
        try invoke(
          document, 17, [["text": "X"], ["Y", "A"], ["text": "Group X", "type": "text"]])
        try invoke(
          document, 17, [["text": "Y"], ["X", "B"], ["text": "Group Y", "type": "text"]])
      }
      graph = try document.snapshot().graph
      let b = try node(graph, "B")
      let oldStyles = try ref(graph.field("styles", on: b))
      var resources = Pele.HostResources()
      resources.textWidth = { request in
        MainActor.assumeIsolated {
          state.trace.append([
            "width", request.text, String(format: "%016llx", request.size.bitPattern),
          ])
          return Double(request.text.utf16.count) * request.size * 0.6
        }
      }
      log.call = { _, args in
        guard args.count == 2, case .string(let level) = args[0],
          case .string(let message) = args[1]
        else { throw PeleError(code: "test", message: "Diagnostic log arguments lost") }
        state.trace.append([
          "log", level.stringReplacingInvalidCodeUnits, message.stringReplacingInvalidCodeUnits,
        ])
        if state.fired { return .undefined }
        state.fired = true
        XCTAssertThrowsError(try document.close())
        var current = try document.snapshot().graph
        switch mode {
        case "title":
          try self.text(&current, "title", "Changed title")
          try document.apply(current, expectedRevision: document.revision)
        case "later-label":
          try self.text(&current, "text", "Changed Beta", on: self.node(current, "B"))
          try document.apply(current, expectedRevision: document.revision)
        case "style-replace":
          let array = try self.input(["fill:red"], &current)
          try current.setField("styles", value: array, on: self.node(current, "B"))
          try document.apply(current, expectedRevision: document.revision)
        case "add-node":
          try self.invoke(document, 3, ["C", ["text": "Gamma", "type": "text"], "task"])
        case "throw-object", "throw-undefined": throw original
        case "nested", "nested-throw":
          state.nested = try document.render(resources: resources)
          if mode == "nested-throw" { throw original }
        default: break
        }
        return .undefined
      }
      let first = try XCTUnwrap(row["first"] as? [String: Any])
      do { try compare(document.render(resources: resources), first, id) } catch {
        XCTAssertEqual(first["ok"] as? Bool, false, id)
        XCTAssertEqual(first["original"] as? Bool, true, id)
        XCTAssertTrue((error as AnyObject) === original, id)
      }
      XCTAssertTrue(state.fired, id)
      XCTAssertEqual(try json(state.trace), try json(XCTUnwrap(row["firstTrace"])), id)
      graph = try document.snapshot().graph
      XCTAssertEqual(
        try observed(graph, oldStyles: oldStyles), try json(XCTUnwrap(row["after"])), id)
      if let nested = state.nested {
        try compare(nested, XCTUnwrap(row["nested"] as? [String: Any]), id + "/nested")
      } else {
        XCTAssertTrue(row["nested"] is NSNull, id)
      }
      try text(&graph, "text", "After cache", on: node(graph, "B"))
      guard case .array(let count, _) = graph.nodes[Int(oldStyles)] else {
        throw ConstructedAgentflowError()
      }
      let stroke = try graph.intern("stroke:blue")
      try graph.setArrayElement(count, value: .string(stroke), on: oldStyles)
      try document.apply(graph, expectedRevision: document.revision)
      state.trace.removeAll()
      try compare(
        document.render(resources: resources), XCTUnwrap(row["recovery"] as? [String: Any]),
        id + "/recovery")
      XCTAssertEqual(try json(state.trace), try json(XCTUnwrap(row["recoveryTrace"])), id)
      let after = try document.snapshot()
      XCTAssertEqual(after.revision, document.revision)
      XCTAssertEqual(
        try observed(after.graph, oldStyles: oldStyles), try json(XCTUnwrap(row["afterRecovery"])),
        id)
    }
  }
}
