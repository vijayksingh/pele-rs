import Foundation
import XCTest
import Pele

final class SVGRootTests: XCTestCase {
    private struct Fixture: Decodable {
        struct Item: Decodable {
            struct Options: Decodable {
                let idPrefix: String?
                let responsive: Bool?
                let fontFamily: String?
                let fontFamilyMono: String?
            }
            struct Drawing: Decodable {
                let type: String
                let svg: String
                let width: Double
                let height: Double
                let links: [DiagramLink]
            }
            let id: String
            let source: String
            let markup: String
            let options: Options
            let result: Drawing
        }
        let cases: [Item]
    }

    func testCompleteSVGRootAcrossPublicRenderLanes() throws {
        let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("../../../..").standardizedFileURL
        let fixture = try JSONDecoder().decode(Fixture.self, from: Data(contentsOf: root.appendingPathComponent("fixtures/pele/svg-root.json")))
        XCTAssertEqual(fixture.cases.count, 115)
        let engine = Pele.Engine()
        for item in fixture.cases {
            var options = Pele.Engine.Options()
            options.idPrefix = item.options.idPrefix
            options.responsive = item.options.responsive
            options.fontFamily = item.options.fontFamily
            options.fontFamilyMono = item.options.fontFamilyMono
            options.now = 1_704_067_200_000
            var host = Pele.HostResources()
            host.textWidth = { request in Double(request.text.utf16.count) * request.size * (request.style & 1 != 0 ? 0.71 : 0.6) }
            host.icon = { _ in Pele.TrustedSVG(trustedMarkup: item.markup) }
            let typed = try engine.parse(item.source)
            let mutable = try engine.parseMutable(item.source)
            defer { try? mutable.close() }
            let drawings = [try engine.render(item.source, options: options, resources: host),
                            try engine.render(typed, options: options, resources: host),
                            try mutable.render(options: options, resources: host)]
            for drawing in drawings {
                XCTAssertEqual(drawing.type, item.result.type, item.id)
                XCTAssertEqual(drawing.svg, item.result.svg, item.id)
                XCTAssertEqual(drawing.width.bitPattern, item.result.width.bitPattern, item.id)
                XCTAssertEqual(drawing.height.bitPattern, item.result.height.bitPattern, item.id)
                XCTAssertEqual(try JSONEncoder().encode(drawing.links), try JSONEncoder().encode(item.result.links), item.id)
            }
        }
    }
}
