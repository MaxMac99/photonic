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
    func user_stats(_ input: Operations.user_stats.Input) async throws
        -> Operations.user_stats.Output
    {
        return .ok(
            .init(
                body: .json(
                    .init(albums: 10, media: 200, quota: 2_000_000_000, quota_used: 17_741_661_000))
            ))
    }

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
        return .created(.init(body: .json(UUID().uuidString)))
    }

    func delete_medium(_ input: Operations.delete_medium.Input) async throws
        -> Operations.delete_medium.Output
    {
        return .noContent(.init(body: .json(try .init())))
    }

    func create_medium(_ input: Operations.create_medium.Input) async throws
        -> Operations.create_medium.Output
    {
        return .created(.init(body: .json(UUID().uuidString)))
    }

    func get_all_media(_ input: Operations.get_all_media.Input) async throws
        -> Operations.get_all_media.Output
    {
        return .ok(
            .init(
                body: .json([
                    .init(
                        id: UUID().uuidString,
                        items: [
                            .init(
                                filename: "IMG_4597", filesize: 12000, id: UUID().uuidString,
                                is_primary: true, last_saved: Date(), medium_item_type: .Original,
                                mime: "image/raw", priority: 0)
                        ], medium_type: .PHOTO)
                ])))
    }

    func system_info(_ input: Operations.system_info.Input) async throws
        -> Operations.system_info.Output
    {
        return .ok(
            .init(
                body: .json(
                    .init(
                        authorize_url: "https://auth.mvissing.de/application/o/authorize/",
                        client_id: "nWqJe0OkoCG5wXXjYdXWWOF78RNFIknlsyKtxHH2",
                        token_url: "https://auth.mvissing.de/application/o/token/", version: "0.1.0"
                    ))))
    }

    func create_album(_ input: Operations.create_album.Input) async throws
        -> Operations.create_album.Output
    {
        return .created(.init(body: .json(UUID().uuidString)))
    }

    func find_all_albums(_ input: Operations.find_all_albums.Input) async throws
        -> Operations.find_all_albums.Output
    {
        return .ok(.init(body: .json(.init())))
    }

}
