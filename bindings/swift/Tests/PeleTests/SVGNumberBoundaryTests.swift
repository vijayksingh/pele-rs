import Foundation
import XCTest
import Pele

final class SVGNumberBoundaryTests: XCTestCase {
    private struct Fixture: Decodable {
        struct Item: Decodable {
            struct Drawing: Decodable { let type: String; let svg: String; let links: [DiagramLink] }
            let id: String
            let paddingBits: String
            let widthBits: String
            let heightBits: String
            let result: Drawing
        }
        let source: String
        let cases: [Item]
    }
    func testRailroadCoordinateBoundariesAcrossAllRenderLanes() throws {
        let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("../../../..").standardizedFileURL
        let fixture = try JSONDecoder().decode(Fixture.self, from: Data(contentsOf: root.appendingPathComponent("fixtures/pele/svg-number-boundaries.json")))
        XCTAssertEqual(fixture.cases.count, 31)
        for item in fixture.cases {
            let engine = Pele.Engine()
            var options = Pele.Engine.Options()
            options.fontSize = 13; options.maxWidth = 220
            options.padding = Double(bitPattern: try XCTUnwrap(UInt64(item.paddingBits, radix: 16)))
            options.now = 1_704_067_200_000
            var host = Pele.HostResources()
            host.textWidth = { request in Double(request.text.utf16.count) * request.size * (request.style & 1 != 0 ? 0.71 : 0.6) }
            let immutable = try engine.parse(fixture.source)
            let mutable = try engine.parseMutable(fixture.source)
            defer { try? mutable.close() }
            let drawings = [try engine.render(fixture.source, options: options, resources: host),
                            try engine.render(immutable, options: options, resources: host),
                            try mutable.render(options: options, resources: host)]
            for drawing in drawings {
                XCTAssertEqual(drawing.type, item.result.type, item.id)
                XCTAssertEqual(drawing.svg, item.result.svg, item.id)
                XCTAssertEqual(drawing.links.map(\.id), item.result.links.map(\.id), item.id)
                for (actual, raw) in [(drawing.width, item.widthBits), (drawing.height, item.heightBits)] {
                    let expected = try XCTUnwrap(UInt64(raw, radix: 16))
                    if Double(bitPattern: expected).isNaN { XCTAssertTrue(actual.isNaN, item.id) }
                    else { XCTAssertEqual(actual.bitPattern, expected, item.id) }
                }
            }
        }
    }
}
