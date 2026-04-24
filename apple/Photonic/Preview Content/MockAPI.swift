//
//  MockAPI.swift
//  Photonic
//
//  Created by Max Vissing on 04.05.24.
//

import Foundation
import OpenAPIRuntime
import OpenAPIURLSession

struct MockAPI: APIProtocol {
    func get_medium_preview(_ input: Operations.get_medium_preview.Input) async throws
        -> Operations.get_medium_preview.Output
    {
        let file = Bundle.main.url(forResource: "IMG_4597", withExtension: "DNG")!
        let data = try Data(contentsOf: file)
        return .ok(.init(body: .any(HTTPBody(data))))
    }

    func get_medium_item(_ input: Operations.get_medium_item.Input) async throws
        -> Operations.get_medium_item.Output
    {
        let file = Bundle.main.url(forResource: "IMG_4597", withExtension: "DNG")!
        let data = try Data(contentsOf: file)
        return .ok(.init(body: .any(HTTPBody(data))))
    }

    func add_medium_item(_ input: Operations.add_medium_item.Input) async throws
        -> Operations.add_medium_item.Output
    {
        .created(.init(body: .json(UUID().uuidString)))
    }

    func delete_medium(_ input: Operations.delete_medium.Input) async throws
        -> Operations.delete_medium.Output
    {
        .noContent(.init())
    }

    func create_medium(_ input: Operations.create_medium.Input) async throws
        -> Operations.create_medium.Output
    {
        .created(.init(body: .json(UUID().uuidString)))
    }

    func get_medium(_ input: Operations.get_medium.Input) async throws
        -> Operations.get_medium.Output
    {
        let now = Date()
        let id = UUID().uuidString
        return .ok(
            .init(
                body: .json(
                    .init(
                        created_at: now,
                        id: id,
                        items: [
                            .init(
                                created_at: now,
                                filename: "IMG_4597",
                                filesize: 12000,
                                id: UUID().uuidString,
                                is_primary: true,
                                medium_item_type: .original,
                                mime: "image/raw",
                                priority: 0
                            )
                        ],
                        medium_type: .PHOTO,
                        updated_at: now
                    )
                )
            )
        )
    }

    func get_medium_metadata(_ input: Operations.get_medium_metadata.Input) async throws
        -> Operations.get_medium_metadata.Output
    {
        .ok(
            .init(
                body: .json(
                    .init(
                        file_info: .init(file_size: 12000, mime_type: "image/raw"),
                        technical: .init()
                    )
                )
            )
        )
    }

    func get_all_media(_ input: Operations.get_all_media.Input) async throws
        -> Operations.get_all_media.Output
    {
        .ok(
            .init(
                body: .json([
                    .init(
                        id: UUID().uuidString,
                        items: [
                            .init(
                                filename: "IMG_4597",
                                filesize: 12000,
                                id: UUID().uuidString,
                                is_primary: true,
                                medium_item_type: .original,
                                mime: "image/raw"
                            )
                        ],
                        medium_type: .PHOTO
                    )
                ])
            )
        )
    }

    func system_info(_ input: Operations.system_info.Input) async throws
        -> Operations.system_info.Output
    {
        .ok(
            .init(
                body: .json(
                    .init(
                        authorize_url: "https://auth.mvissing.de/application/o/authorize/",
                        client_id: "nWqJe0OkoCG5wXXjYdXWWOF78RNFIknlsyKtxHH2",
                        token_url: "https://auth.mvissing.de/application/o/token/",
                        version: "0.1.0"
                    )
                )
            )
        )
    }
}
