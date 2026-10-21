import Foundation
import Pele
import XCTest

private final class GanttRenderFailure: Error {}
@MainActor private enum GanttRenderDocument {
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
  func render(_ resources: Pele.HostResources, responsive: Bool) throws -> FullRenderedDiagram {
    var options = Pele.Engine.Options()
    options.maxWidth = responsive ? 320 : nil
    options.now = 1_704_283_200_000
    switch self {
    case .normal(let d): return try d.render(options: options, resources: resources)
    case .prepared(let d): return try d.render(options: options, resources: resources)
    }
  }
  func close() throws {
    switch self {
    case .normal(let d): try d.close()
    case .prepared(let d): try d.close()
    }
  }
}
@MainActor private final class GanttRenderState {
  var fired = false
  var trace: [[Any]] = []
  var nested: FullRenderedDiagram?
  var nestedRevision: UInt64?
  let failure = GanttRenderFailure()
}
@MainActor final class NativeGanttCallbackTests: XCTestCase {
  private func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
  private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
    XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String))
    XCTAssertEqual(bits(actual.width), try XCTUnwrap(expected["width"] as? String))
    XCTAssertEqual(bits(actual.height), try XCTUnwrap(expected["height"] as? String))
    let links = try XCTUnwrap(expected["links"] as? [[String: Any]])
    let actualLinks: [[String: Any]] = actual.links.map {
      ["id": $0.id, "href": $0.href, "internal": $0.internal]
    }
    XCTAssertTrue(NSArray(array: actualLinks).isEqual(to: links))
  }
  private static func resources() -> Pele.HostResources {
    var host = Pele.HostResources()
    host.timeZone = TimeZone(secondsFromGMT: 0)!
    host.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
    return host
  }
  private static func ref(_ atom: Pele.PublicGraph.Atom?) throws -> UInt32 {
    guard case .ref(let id) = atom else {
      throw PeleError(code: "test", message: "Gantt reference missing")
    }
    return id
  }
  private static func text(
    _ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil
  ) throws {
    let id = try graph.intern(value)
    try graph.setField(key, value: .string(id), on: node)
  }
  private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom])
    throws -> Pele.PublicGraph.Atom
  {
    let id = UInt32(graph.nodes.count)
    var properties: [Pele.PublicGraph.Property] = []
    for (i, value) in values.enumerated() {
      properties.append(.init(key: try graph.intern(String(i)), value: value))
    }
    graph.nodes.append(.array(length: UInt32(values.count), properties: properties))
    return .ref(id)
  }
  private static func append(
    _ graph: inout Pele.PublicGraph, _ value: Pele.PublicGraph.Atom, on id: UInt32
  ) throws {
    guard case .array(let length, _) = graph.nodes[Int(id)] else {
      throw PeleError(code: "test", message: "Gantt array missing")
    }
    try graph.setField(String(length), value: value, on: id)
  }
  private static func date(_ graph: inout Pele.PublicGraph, _ milliseconds: Double)
    -> Pele.PublicGraph.Atom
  {
    let id = UInt32(graph.nodes.count)
    graph.nodes.append(.date(milliseconds: milliseconds, properties: []))
    return .ref(id)
  }
  private static func copyTask(
    _ graph: inout Pele.PublicGraph, _ source: UInt32, id: String, label: String
  ) throws -> Pele.PublicGraph.Atom {
    let node = UInt32(graph.nodes.count)
    graph.nodes.append(.object(properties: graph.nodes[Int(source)].properties))
    try text(&graph, "id", id, on: node)
    try text(&graph, "task", label, on: node)
    let old = try ref(graph.field("classes", on: source))
    guard case .array(let length, _) = graph.nodes[Int(old)] else {
      throw PeleError(code: "test", message: "Gantt classes missing")
    }
    let values = try (0..<length).map { try graph.field(String($0), on: old) ?? .undefined }
    let classes = try array(&graph, values)
    try graph.setField("classes", value: classes, on: node)
    return .ref(node)
  }
  private static func value(_ graph: Pele.PublicGraph, _ atom: Pele.PublicGraph.Atom) throws -> Any
  {
    switch atom {
    case .undefined, .null: return NSNull()
    case .bool(let value): return value
    case .number(let value): return value.isFinite ? value as Any : NSNull()
    case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
    case .ref(let id):
      let node = graph.nodes[Int(id)]
      if case .date(let ms, _) = node {
        guard ms.isFinite else { return NSNull() }
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        return formatter.string(from: Date(timeIntervalSince1970: ms / 1000))
      }
      if case .array(let length, _) = node {
        return try (0..<length).map {
          try value(graph, graph.field(String($0), on: id) ?? .undefined)
        }
      }
      if case .map(let entries, _) = node {
        return try entries.map { [try value(graph, $0.key), try value(graph, $0.value)] }
      }
      var object: [String: Any] = [:]
      for property in node.properties {
        if case .undefined = property.value { continue }
        object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(
          graph, property.value)
      }
      return object
    default: throw PeleError(code: "test", message: "Unexpected Gantt fixture atom")
    }
  }
  private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
    var result: [String: Any] = [:]
    for key in [
      "title", "accTitle", "accDescr", "displayMode", "topAxis", "axisFormat", "dateFormat",
      "tickInterval", "todayMarker", "tasks", "sections", "links", "includes", "excludes", "work",
      "now",
    ] {
      let atom = try graph.field(key) ?? .undefined
      if case .undefined = atom, !["title", "accTitle", "accDescr", "tickInterval"].contains(key) {
        continue
      }
      result[key] = try value(graph, atom)
    }
    return result
  }
  func testGanttLiveSchedulingDatesCallbacksAndRecoveryMatchAllOracleRows() throws {
    var path = URL(fileURLWithPath: #filePath)
    for _ in 0..<5 { path.deleteLastPathComponent() }
    let fixture = try XCTUnwrap(
      JSONSerialization.jsonObject(
        with: Data(
          contentsOf: path.appendingPathComponent("fixtures/public-gantt-live-render.json")))
        as? [String: Any])
    XCTAssertEqual(
      fixture["oracleSHA256"] as? String,
      "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
    let source = try XCTUnwrap(fixture["source"] as? String)
    let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
    XCTAssertEqual(rows.count, 72)
    for row in rows {
      let raw = try XCTUnwrap(row["raw"] as? Bool)
      let responsive = try XCTUnwrap(row["responsive"] as? Bool)
      let mode = try XCTUnwrap(row["mode"] as? String)
      let engine = Pele.Engine()
      let document: GanttRenderDocument =
        try raw
        ? .prepared(engine.parsePrepared("gantt", source: source))
        : .normal(engine.parseMutable(source))
      defer { try? document.close() }
      var setup = try document.snapshot().graph
      if responsive {
        try Self.text(&setup, "displayMode", "compact")
        try setup.setField("topAxis", value: .bool(true))
        try document.apply(setup)
      }
      let tasks = try Self.ref(setup.field("tasks"))
      let a = try Self.ref(setup.field("0", on: tasks))
      let b = try Self.ref(setup.field("1", on: tasks))
      let c = try Self.ref(setup.field("2", on: tasks))
      let observed = GanttRenderState()
      var resources = Self.resources()
      resources.textWidth = { request in
        try MainActor.assumeIsolated {
          observed.trace.append([
            "width", request.text, String(format: "%016llx", request.size.bitPattern),
            Int(request.style),
          ])
          if !observed.fired && request.text == "Alpha" {
            observed.fired = true
            XCTAssertThrowsError(try document.close())
            var graph = try document.snapshot().graph
            switch mode {
            case "current-label": try Self.text(&graph, "task", "Changed current", on: a)
            case "next-label": try Self.text(&graph, "task", "Changed later", on: b)
            case "dates":
              for (id, key, ms) in [
                (a, "startTime", 1704153600000.0), (a, "endTime", 1704326400000.0),
                (b, "endTime", 1704585600000.0),
              ] {
                let d = Self.date(&graph, ms)
                try graph.setField(key, value: d, on: id)
              }
            case "render-end":
              let d = Self.date(&graph, 1_704_153_600_000)
              try graph.setField("renderEndTime", value: d, on: a)
            case "flags":
              for key in ["active", "crit", "milestone"] {
                try graph.setField(key, value: .bool(true), on: a)
              }
              try graph.setField("done", value: .bool(true), on: b)
            case "classes":
              let classes = try Self.ref(graph.field("classes", on: a))
              let s = try graph.intern("clickable")
              try Self.append(&graph, .string(s), on: classes)
            case "links":
              let id = try Self.ref(graph.field("links"))
              let key = try graph.intern("a")
              let href = try graph.intern("https://example.com/alpha")
              guard case .map(var entries, let properties) = graph.nodes[Int(id)] else {
                throw PeleError(code: "test", message: "Gantt links missing")
              }
              entries.append(.init(key: .string(key), value: .string(href)))
              graph.nodes[Int(id)] = .map(entries: entries, properties: properties)
            case "tasks-replace":
              let task = try Self.copyTask(&graph, b, id: "replacement", label: "Replacement")
              let array = try Self.array(&graph, [task])
              try graph.setField("tasks", value: array)
            case "tasks-append":
              let task = try Self.copyTask(&graph, c, id: "appended", label: "Appended")
              try graph.setField("order", value: .number(3), on: Self.ref(task))
              try Self.append(&graph, task, on: tasks)
            case "section":
              try Self.text(&graph, "section", "Changed current section", on: a)
              try Self.text(&graph, "section", "Changed next section", on: b)
            case "axis":
              try Self.text(&graph, "axisFormat", "%d")
              try Self.text(&graph, "tickInterval", "1day")
            case "title-accessibility":
              try Self.text(&graph, "title", "Changed title")
              try Self.text(&graph, "accTitle", "Changed access")
              try Self.text(&graph, "accDescr", "Changed description")
            case "today": try Self.text(&graph, "todayMarker", "off")
            case "exclusions":
              let day = try graph.intern("2024-01-02")
              let values: [Pele.PublicGraph.Atom] = [.string(day)]
              let exclusions = try Self.array(&graph, values)
              let set = UInt32(graph.nodes.count)
              graph.nodes.append(.set(values: values, properties: []))
              try graph.setField("excludes", value: exclusions)
              try graph.setField("excludeSet", value: .ref(set))
              try graph.setField("datedTokens", value: .bool(true))
            case "throw-object", "throw-undefined":
              try Self.text(&graph, "title", "Before failure")
              try Self.text(&graph, "task", "Before failure task", on: b)
            case "nested": try Self.text(&graph, "title", "Nested title")
            case "baseline": break
            default: throw PeleError(code: "test", message: "Unknown Gantt mode")
            }
            if mode != "baseline" { try document.apply(graph) }
            if mode.hasPrefix("throw-") { throw observed.failure }
            if mode == "nested" {
              observed.nested = try document.render(Self.resources(), responsive: responsive)
              observed.nestedRevision = document.revision
              var latest = try document.snapshot().graph
              try Self.text(&latest, "title", "After nested")
              try document.apply(latest)
            }
          }
          return Double(request.text.utf16.count) * request.size * 0.6
        }
      }
      let first = try XCTUnwrap(row["first"] as? [String: Any])
      if mode.hasPrefix("throw-") {
        XCTAssertEqual(first["original"] as? Bool, true)
        XCTAssertThrowsError(try document.render(resources, responsive: responsive)) {
          XCTAssertTrue(($0 as AnyObject) === observed.failure)
        }
      } else {
        try compare(document.render(resources, responsive: responsive), first)
      }
      XCTAssertTrue(observed.fired, mode)
      XCTAssertTrue(
        NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])),
        mode + " trace")
      let after = try document.snapshot()
      XCTAssertEqual(after.revision, document.revision)
      XCTAssertTrue(
        NSDictionary(dictionary: try Self.state(after.graph)).isEqual(
          to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
      let original = try XCTUnwrap(row["original"] as? [String: Any])
      for (key, id) in [("tasks", tasks), ("a", a), ("b", b), ("c", c)] {
        let actual = try Self.value(after.graph, .ref(id))
        let expected = try XCTUnwrap(original[key])
        XCTAssertEqual(
          try JSONSerialization.data(withJSONObject: [actual], options: [.sortedKeys]),
          try JSONSerialization.data(withJSONObject: [expected], options: [.sortedKeys]), mode + key
        )
      }
      let current = try Self.ref(after.graph.field("tasks"))
      let identity = try XCTUnwrap(row["identity"] as? [String: Bool])
      XCTAssertEqual(current == tasks, identity["tasks"])
      guard case .array(let length, _) = after.graph.nodes[Int(current)] else {
        return XCTFail("Gantt tasks missing")
      }
      let currentIDs = try (0..<length).compactMap { index -> UInt32? in
        if case .ref(let id) = try after.graph.field(String(index), on: current) { return id }
        return nil
      }
      for (key, id) in [("a", a), ("b", b), ("c", c)] {
        XCTAssertEqual(currentIDs.contains(id), identity[key])
      }
      if mode == "nested" {
        try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any]))
        XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision))
      }
      let recovered = GanttRenderState()
      var plain = Self.resources()
      plain.textWidth = { request in
        MainActor.assumeIsolated {
          recovered.trace.append([
            "width", request.text, String(format: "%016llx", request.size.bitPattern),
            Int(request.style),
          ])
          return Double(request.text.utf16.count) * request.size * 0.6
        }
      }
      try compare(
        document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any])
      )
      XCTAssertTrue(
        NSArray(array: recovered.trace).isEqual(
          to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
    }
  }
}
