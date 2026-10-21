import Foundation
import Pele
import XCTest

private final class ClassCallbackFailure: Error {}
@MainActor private enum ClassCallbackDocument {
  case normal(Pele.MutableDocument)
  case prepared(Pele.PreparedDocument)
  var revision: UInt64 {
    switch self {
    case .normal(let d): d.revision
    case .prepared(let d): d.revision
    }
  }
  func snapshot() throws -> Pele.PublicSnapshot {
    switch self {
    case .normal(let d): try d.snapshot()
    case .prepared(let d): try d.snapshot()
    }
  }
  func apply(_ graph: Pele.PublicGraph) throws {
    switch self {
    case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision)
    case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision)
    }
  }
  func apply(_ graph: Pele.PublicGraph, expectedRevision: UInt64) throws {
    switch self {
    case .normal(let d): _ = try d.apply(graph, expectedRevision: expectedRevision)
    case .prepared(let d): _ = try d.apply(graph, expectedRevision: expectedRevision)
    }
  }
  func render(_ resources: Pele.HostResources, responsive: Bool) throws -> FullRenderedDiagram {
    var options = Pele.Engine.Options()
    options.maxWidth = responsive ? 180 : nil
    switch self {
    case .normal(let d): return try d.render(options: options, resources: resources)
    case .prepared(let d): return try d.render(options: options, resources: resources)
    }
  }
  func bind(_ function: Pele.HostFunction) throws -> Pele.PublicGraph.Atom {
    switch self {
    case .normal(let d): return try d.bindHostFunction(function)
    case .prepared(let d): return try d.bindHostFunction(function)
    }
  }
  func close() throws {
    switch self {
    case .normal(let d): try d.close()
    case .prepared(let d): try d.close()
    }
  }
}
@MainActor private final class ClassCallbackState {
  var fired = false
  var trace: [[Any]] = []
  var nested: FullRenderedDiagram?
  var nestedRevision: UInt64?
  var inside = false
  var calls: [[Any]] = []
  var functions: [Pele.HostFunction] = []
  var escapedViews: [Pele.ActiveModel] = []
  let failure = ClassCallbackFailure()
}
@MainActor final class NativeClassCallbackTests: XCTestCase {
  private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
  private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
    guard case .ref(let id) = value else {
      throw PeleError(code: "test", message: "Expected Class reference")
    }
    return id
  }
  private static func mapValue(_ graph: Pele.PublicGraph, _ key: String, _ name: String) throws
    -> UInt32
  {
    let map = try ref(graph.field(key))
    guard case .map(let entries, _) = graph.nodes[Int(map)] else { throw ClassCallbackFailure() }
    for entry in entries {
      if case .string(let id) = entry.key, graph.strings[Int(id)].stringIfWellFormed == name {
        return try ref(entry.value)
      }
    }
    throw ClassCallbackFailure()
  }
  private static func nodes(_ g: Pele.PublicGraph) throws -> (
    classes: UInt32, a: UInt32, b: UInt32, members: UInt32, member: UInt32, second: UInt32,
    relations: UInt32
  ) {
    let a = try mapValue(g, "classes", "A")
    let b = try mapValue(g, "classes", "B")
    let members = try ref(g.field("members", on: a))
    return (
      try ref(g.field("classes")), a, b, members, try ref(g.field("0", on: members)),
      try ref(g.field("1", on: members)), try ref(g.field("relations"))
    )
  }
  private static func setText(
    _ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil
  ) throws {
    let id = try graph.intern(value)
    try graph.setField(key, value: .string(id), on: node)
  }
  private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom])
    throws -> UInt32
  {
    let id = UInt32(graph.nodes.count)
    graph.nodes.append(.array(length: 0, properties: []))
    for (i, value) in values.enumerated() {
      try graph.setArrayElement(UInt32(i), value: value, on: id)
    }
    return id
  }
  private static func encoded(_ graph: Pele.PublicGraph, _ atom: Pele.PublicGraph.Atom) throws
    -> Any
  {
    switch atom {
    case .undefined: return NSNull()
    case .null: return NSNull()
    case .bool(let b): return b
    case .number(let n): return n.isFinite ? n : NSNull()
    case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
    case .ref(let id):
      if case .array(let length, _) = graph.nodes[Int(id)] {
        return try (0..<length).map {
          try encoded(graph, graph.field(String($0), on: id) ?? .undefined)
        }
      }
      var object: [String: Any] = [:]
      for p in graph.nodes[Int(id)].properties {
        if case .undefined = p.value { continue }
        if case .hostReference = p.value { continue }
        if case .builtinFunction = p.value { continue }
        object[try XCTUnwrap(graph.strings[Int(p.key)].stringIfWellFormed)] = try encoded(
          graph, p.value)
      }
      return object
    default: throw ClassCallbackFailure()
    }
  }
  private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
    var result: [String: Any] = [:]
    for key in [
      "title", "accTitle", "accDescr", "direction", "classes", "relations", "notes", "namespaces",
      "interfaces", "styleClasses",
    ] {
      let value = try graph.field(key) ?? .undefined
      if case .undefined = value { continue }
      if ["classes", "namespaces", "styleClasses"].contains(key) {
        let id = try ref(value)
        guard case .map(let entries, _) = graph.nodes[Int(id)] else { throw ClassCallbackFailure() }
        result[key] = try entries.map {
          [try encoded(graph, $0.key), try encoded(graph, $0.value)]
        }
      } else {
        result[key] = try encoded(graph, value)
      }
    }
    return result
  }
  private static func resources(_ state: ClassCallbackState? = nil) -> Pele.HostResources {
    var host = Pele.HostResources()
    host.textWidth = { request in
      MainActor.assumeIsolated {
        state?.trace.append([request.text, bits(request.size), Int(request.style)])
        return Double(request.text.utf16.count) * request.size * 0.6
      }
    }
    return host
  }
  private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
    XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String))
    XCTAssertEqual(Self.bits(actual.width), expected["width"] as? String)
    XCTAssertEqual(Self.bits(actual.height), expected["height"] as? String)
    XCTAssertTrue(
      NSArray(
        array: actual.links.map {
          ["id": $0.id, "href": $0.href, "internal": $0.internal] as [String: Any]
        }
      ).isEqual(to: try XCTUnwrap(expected["links"] as? [[String: Any]])))
  }
  private static func mutate(
    _ document: ClassCallbackDocument, _ observed: ClassCallbackState, _ mode: String,
    _ responsive: Bool
  ) throws {
    observed.fired = true
    XCTAssertThrowsError(try document.close())
    var graph = try document.snapshot().graph
    let n = try nodes(graph)
    switch mode {
    case "baseline": break
    case "title": try setText(&graph, "title", "Changed")
    case "accessibility":
      try setText(&graph, "accTitle", "Changed access")
      try setText(&graph, "accDescr", "Changed description")
    case "later-label": try setText(&graph, "label", "Changed beta", on: n.b)
    case "annotations":
      let id = try ref(graph.field("annotations", on: n.a))
      guard case .array(let length, _) = graph.nodes[Int(id)] else { throw ClassCallbackFailure() }
      let text = try graph.intern("Changed annotation")
      try graph.setArrayElement(length, value: .string(text), on: id)
    case "later-member": try setText(&graph, "id", "changed: bool", on: n.second)
    case "append-member":
      let id = UInt32(graph.nodes.count)
      graph.nodes.append(graph.nodes[Int(n.member)])
      try setText(&graph, "id", "appended: int", on: id)
      guard case .array(let length, _) = graph.nodes[Int(n.members)] else {
        throw ClassCallbackFailure()
      }
      try graph.setArrayElement(length, value: .ref(id), on: n.members)
    case "member-classifier", "width-classifier":
      try setText(&graph, "classifier", "*", on: n.member)
    case "replace-classes":
      let id = UInt32(graph.nodes.count)
      let key = try graph.intern("B")
      graph.nodes.append(
        .map(entries: [.init(key: .string(key), value: .ref(n.b))], properties: []))
      try graph.setField("classes", value: .ref(id))
    case "relation-label":
      let edge = try ref(graph.field("0", on: n.relations))
      try setText(&graph, "title", "Changed edge", on: edge)
    case "late-tooltip":
      try setText(&graph, "tooltip", "Changed tooltip", on: n.a)
      try setText(&graph, "link", "https://example.test/class", on: n.a)
      try setText(&graph, "linkTarget", "_blank", on: n.a)
    case "throw-object", "throw-undefined": try setText(&graph, "title", "Saved")
    case "nested": try setText(&graph, "title", "Nested")
    case "replace-display", "replace-classifier", "display-throw", "classifier-throw",
      "display-reentry":
      let classifier = mode.contains("classifier")
      let function = Pele.HostFunction { view, receiver, args in
        observed.escapedViews.append(view)
        XCTAssertThrowsError(try document.close())
        XCTAssertTrue(args.isEmpty)
        observed.calls.append([
          classifier ? "classifier" : "display", try ref(receiver) == n.member,
        ])
        let root = try view.root()
        if mode == "display-throw" || mode == "classifier-throw" {
          try view.set(
            "title", on: root,
            to: view.intern(mode == "display-throw" ? "Saved display" : "Saved classifier"))
          throw observed.failure
        }
        if classifier {
          try view.set("id", on: receiver, to: view.intern("changed in classifier"))
          return try view.intern("font-style:italic;")
        }
        if mode == "display-reentry" && !observed.inside {
          observed.inside = true
          defer { observed.inside = false }
          observed.nested = try document.render(resources(), responsive: responsive)
          observed.nestedRevision = document.revision
        }
        let out = try view.object()
        try view.set(
          "displayText", on: out,
          to: view.intern(mode == "display-reentry" ? "custom nested" : "custom display"))
        try view.set("cssStyle", on: out, to: view.intern(""))
        return out
      }
      observed.functions.append(function)
      let value = try document.bind(function)
      try graph.setField(
        classifier ? "parseClassifier" : "getDisplayDetails", value: value, on: n.member)
    default: throw ClassCallbackFailure()
    }
    if mode != "baseline" { try document.apply(graph) }
    if mode.hasPrefix("throw-") { throw observed.failure }
    if mode == "nested" {
      observed.nested = try document.render(resources(), responsive: responsive)
      observed.nestedRevision = document.revision
      var latest = try document.snapshot().graph
      try setText(&latest, "title", "After nested")
      try document.apply(latest)
    }
  }
  func testClassOwnedResourceAndMemberCallbacksMatch80PinnedOracleRows() throws {
    var path = URL(fileURLWithPath: #filePath)
    for _ in 0..<5 { path.deleteLastPathComponent() }
    let fixture = try XCTUnwrap(
      JSONSerialization.jsonObject(
        with: Data(
          contentsOf: path.appendingPathComponent("fixtures/public-class-live-render.json")))
        as? [String: Any])
    XCTAssertEqual(
      fixture["oracleSHA256"] as? String,
      "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
    let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
    let source = try XCTUnwrap(fixture["source"] as? String)
    XCTAssertEqual(rows.count, 80)
    let identities = rows.map { "\($0["raw"]!)|\($0["responsive"]!)|\($0["mode"]!)" }
    XCTAssertEqual(Set(identities).count, 80, "Every fixture scenario must execute once")
    for row in rows {
      let raw = try XCTUnwrap(row["raw"] as? Bool)
      let responsive = try XCTUnwrap(row["responsive"] as? Bool)
      let mode = try XCTUnwrap(row["mode"] as? String)
      let engine = Pele.Engine()
      let document: ClassCallbackDocument =
        try raw
        ? .prepared(engine.parsePrepared("class", source: source))
        : .normal(engine.parseMutable(source))
      defer { try? document.close() }
      var initial = try document.snapshot().graph
      let original = try Self.nodes(initial)
      try Self.setText(&initial, "title", "Initial")
      try Self.setText(&initial, "label", "Alpha", on: original.a)
      try Self.setText(&initial, "label", "Beta", on: original.b)
      try document.apply(initial)
      let observed = ClassCallbackState()
      var host = Self.resources()
      host.textWidth = { request in
        try MainActor.assumeIsolated {
          observed.trace.append([request.text, Self.bits(request.size), Int(request.style)])
          if !observed.fired
            && (mode == "width-classifier"
              ? request.text.contains("first") : request.text == "Alpha")
          {
            try Self.mutate(document, observed, mode, responsive)
          }
          return Double(request.text.utf16.count) * request.size * 0.6
        }
      }
      let first = try XCTUnwrap(row["first"] as? [String: Any])
      if first["ok"] as? Bool == false {
        XCTAssertEqual(first["original"] as? Bool, true)
        XCTAssertThrowsError(try document.render(host, responsive: responsive)) {
          XCTAssertTrue(($0 as AnyObject) === observed.failure)
        }
      } else {
        try compare(document.render(host, responsive: responsive), first)
      }
      XCTAssertTrue(observed.fired, mode)
      XCTAssertTrue(
        NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])),
        mode + " trace")
      XCTAssertTrue(
        NSArray(array: observed.calls).isEqual(to: try XCTUnwrap(row["calls"] as? [[Any]])),
        mode + " calls")
      let after = try document.snapshot()
      XCTAssertEqual(after.revision, document.revision)
      XCTAssertTrue(
        NSDictionary(dictionary: try Self.state(after.graph)).isEqual(
          to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
      let aliases = try XCTUnwrap(row["aliases"] as? [String: Bool])
      XCTAssertEqual(
        try Self.ref(after.graph.field("classes")) == original.classes, aliases["classes"])
      let currentA: UInt32? = try? Self.mapValue(after.graph, "classes", "A")
      XCTAssertEqual(currentA == original.a, aliases["a"])
      XCTAssertEqual(
        try Self.ref(after.graph.field("members", on: original.a)) == original.members,
        aliases["members"])
      XCTAssertEqual(
        try Self.ref(after.graph.field("0", on: original.members)) == original.member,
        aliases["member"])
      XCTAssertEqual(
        try Self.ref(after.graph.field("relations")) == original.relations, aliases["relations"])
      let old = try XCTUnwrap(row["original"] as? [String: Any])
      for (key, id) in [
        ("a", original.a), ("b", original.b), ("member", original.member),
        ("second", original.second),
      ] {
        XCTAssertTrue(
          NSDictionary(
            dictionary: try XCTUnwrap(Self.encoded(after.graph, .ref(id)) as? [String: Any])
          ).isEqual(to: try XCTUnwrap(old[key] as? [String: Any])), mode + key)
      }
      if let nested = row["nested"] as? [String: Any] {
        try compare(XCTUnwrap(observed.nested), nested)
      }
      if let revision = observed.nestedRevision { XCTAssertGreaterThan(after.revision, revision) }
      for view in observed.escapedViews { XCTAssertThrowsError(try view.root()) }

      var repaired = after.graph
      if case .instance(let prototype, let properties) = repaired.nodes[Int(original.member)] {
        repaired.nodes[Int(original.member)] = .instance(
          prototype: prototype,
          properties: properties.filter {
            !["getDisplayDetails", "parseClassifier"].contains(
              repaired.strings[Int($0.key)].stringIfWellFormed ?? "")
          })
      }
      try document.apply(repaired)
      let recovery = try XCTUnwrap(row["recovery"] as? [String: Any])
      XCTAssertEqual(recovery["ok"] as? Bool, true)
      try compare(document.render(Self.resources(), responsive: responsive), recovery)
    }
  }
  func testMemberCallbackNativeWritesPermitNestedRenderButKeepExplicitStaleApply() throws {
    let source = "classDiagram\nclass A {\n+String first\n+String second\n}\nclass B\nA --> B"
    for raw in [false, true] {
      let engine = Pele.Engine()
      let document: ClassCallbackDocument = try raw
        ? .prepared(engine.parsePrepared("class", source: source))
        : .normal(engine.parseMutable(source))
      defer { try? document.close() }
      var initial = try document.snapshot().graph
      let member = try Self.nodes(initial).member
      let observed = ClassCallbackState()
      var nestedCount = 0
      let display = Pele.HostFunction { view, _, _ in
        if !observed.inside {
          observed.inside = true
          defer { observed.inside = false }
          let before = try document.snapshot()
          // This changes native revision without updating Swift's owner cache.
          _ = try view.intern("nested revision marker")
          let nested = try document.render(Self.resources(), responsive: false)
          XCTAssertFalse(nested.svg.isEmpty)
          XCTAssertGreaterThan(document.revision, before.revision)
          nestedCount += 1
          // The implicit refresh must never replace an explicit stale revision.
          XCTAssertThrowsError(try document.apply(before.graph, expectedRevision: before.revision)) {
            XCTAssertEqual(($0 as? PeleError)?.message, "Stale public model revision")
          }
        }
        let output = try view.object()
        try view.set("displayText", on: output, to: view.intern("custom display"))
        try view.set("cssStyle", on: output, to: view.intern(""))
        return output
      }
      observed.functions.append(display)
      try initial.setField("getDisplayDetails", value: document.bind(display), on: member)
      try document.apply(initial)
      let rendered = try document.render(Self.resources(), responsive: false)
      XCTAssertFalse(rendered.svg.isEmpty)
      XCTAssertEqual(nestedCount, 1)
      let acknowledged = document.revision
      XCTAssertEqual(try document.snapshot().revision, acknowledged)
      XCTAssertFalse(try document.render(Self.resources(), responsive: false).svg.isEmpty)
      XCTAssertEqual(nestedCount, 2)
    }
  }

}
