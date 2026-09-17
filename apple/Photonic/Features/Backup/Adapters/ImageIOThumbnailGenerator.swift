import CoreGraphics
import Dependencies
import Foundation
import ImageIO
import PhotonicCore
import UniformTypeIdentifiers

/// A JPEG-encoded thumbnail with its pixel dimensions.
private struct EncodedThumbnail {
    let data: Data
    let width: Int
    let height: Int
}

/// Live implementation: generates the tiny/small/large thumbnail variants
/// from an original image using ImageIO's fast downsampling path, encoding
/// each as JPEG.
struct ImageIOThumbnailGenerator: Sendable {
    private static let compressionQuality: Double = 0.8

    func generate(from imageData: Data) throws -> [GeneratedThumbnail] {
        let sourceOptions = [
            kCGImageSourceShouldCache: false
        ] as CFDictionary

        guard
            let source = CGImageSourceCreateWithData(
                imageData as CFData,
                sourceOptions
            )
        else {
            throw ThumbnailGenerationError.invalidImageData
        }

        return try ThumbnailVariant.allCases.map { variant in
            let thumbnailOptions = [
                kCGImageSourceCreateThumbnailFromImageAlways: true,
                kCGImageSourceCreateThumbnailWithTransform: true,
                kCGImageSourceThumbnailMaxPixelSize: variant.maxPixelSize,
                kCGImageSourceShouldCacheImmediately: true
            ] as CFDictionary

            guard
                let thumbnail = CGImageSourceCreateThumbnailAtIndex(
                    source,
                    0,
                    thumbnailOptions
                )
            else {
                throw ThumbnailGenerationError.invalidImageData
            }

            let encoded = try Self.encodeJPEG(thumbnail)
            return GeneratedThumbnail(
                variant: variant,
                data: encoded.data,
                width: encoded.width,
                height: encoded.height
            )
        }
    }

    private static func encodeJPEG(_ image: CGImage) throws -> EncodedThumbnail {
        let data = NSMutableData()

        guard
            let destination = CGImageDestinationCreateWithData(
                data,
                UTType.jpeg.identifier as CFString,
                1,
                nil
            )
        else {
            throw ThumbnailGenerationError.encodingFailed
        }

        let properties = [
            kCGImageDestinationLossyCompressionQuality: compressionQuality
        ] as CFDictionary

        CGImageDestinationAddImage(destination, image, properties)

        guard CGImageDestinationFinalize(destination) else {
            throw ThumbnailGenerationError.encodingFailed
        }

        return EncodedThumbnail(data: data as Data, width: image.width, height: image.height)
    }
}

enum ThumbnailGenerationError: Error, Equatable {
    case invalidImageData
    case encodingFailed
}

extension ThumbnailGenerator: DependencyKey {
    static var liveValue: ThumbnailGenerator {
        ThumbnailGenerator(generate: { data in
            try ImageIOThumbnailGenerator().generate(from: data)
        })
    }
}
