import CoreGraphics
import ImageIO
import PhotonicCore
import UniformTypeIdentifiers
import XCTest
@testable import Photonic

final class ThumbnailTests: XCTestCase {
    // MARK: - ThumbnailVariant

    func testThumbnailVariantMaxPixelSize() {
        XCTAssertEqual(ThumbnailVariant.tiny.maxPixelSize, 200)
        XCTAssertEqual(ThumbnailVariant.small.maxPixelSize, 720)
        XCTAssertEqual(ThumbnailVariant.large.maxPixelSize, 1440)
    }

    func testThumbnailVariantRawValuesMatchServerContract() {
        XCTAssertEqual(ThumbnailVariant(rawValue: "tiny"), .tiny)
        XCTAssertEqual(ThumbnailVariant(rawValue: "small"), .small)
        XCTAssertEqual(ThumbnailVariant(rawValue: "large"), .large)
    }

    // MARK: - ImageIOThumbnailGenerator

    func testGenerateThumbnailsProducesAllVariantsWithinBounds() throws {
        let generator = ImageIOThumbnailGenerator()
        let original = try makeTestImage(width: 3000, height: 2000)

        let thumbnails = try generator.generate(from: original)

        XCTAssertEqual(thumbnails.map(\.variant), ThumbnailVariant.allCases)
        for thumbnail in thumbnails {
            XCTAssertEqual(
                max(thumbnail.width, thumbnail.height),
                thumbnail.variant.maxPixelSize,
                "Longest edge should match the variant limit"
            )
            XCTAssertFalse(thumbnail.data.isEmpty)
        }
    }

    func testGenerateThumbnailsRejectsInvalidData() {
        let generator = ImageIOThumbnailGenerator()

        XCTAssertThrowsError(try generator.generate(from: Data([0xDE, 0xAD]))) { error in
            XCTAssertEqual(error as? ThumbnailGenerationError, .invalidImageData)
        }
    }

    // MARK: - MediumType.isImage

    func testImageTypesGenerateThumbnails() {
        XCTAssertTrue(MediumType.photo.isImage)
        XCTAssertTrue(MediumType.livePhoto.isImage)
        XCTAssertTrue(MediumType.gif.isImage)
        XCTAssertFalse(MediumType.video.isImage)
        XCTAssertFalse(MediumType.vector.isImage)
    }

    // MARK: - Helpers

    private func makeTestImage(width: Int, height: Int) throws -> Data {
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let context = try XCTUnwrap(
            CGContext(
                data: nil,
                width: width,
                height: height,
                bitsPerComponent: 8,
                bytesPerRow: 0,
                space: colorSpace,
                bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            )
        )

        context.setFillColor(CGColor(red: 0.5, green: 0.25, blue: 0.75, alpha: 1.0))
        context.fill(CGRect(x: 0, y: 0, width: width, height: height))

        let image = try XCTUnwrap(context.makeImage())

        let data = NSMutableData()
        let destination = try XCTUnwrap(
            CGImageDestinationCreateWithData(
                data,
                UTType.jpeg.identifier as CFString,
                1,
                nil
            )
        )
        CGImageDestinationAddImage(destination, image, nil)
        guard CGImageDestinationFinalize(destination) else {
            throw ThumbnailGenerationError.encodingFailed
        }

        return data as Data
    }
}
