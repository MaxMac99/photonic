import PhotonicCore

/// Maps generated system-info payloads to Core models. OAuth endpoints are
/// optional (server may have OIDC disabled) but validated when present (R9).
public enum ServerInfoMapper {
    public static func map(_ payload: Components.Schemas.InfoResponse) throws -> ServerInfo {
        let tokenURL = payload.token_url.flatMap(ServerURL.init)
        let authorizeURL = payload.authorize_url.flatMap(ServerURL.init)
        let hasClientID = !(payload.client_id ?? "").isEmpty

        switch (hasClientID, tokenURL, authorizeURL) {
        case (false, nil, nil), (true, .some, .some):
            break
        default:
            throw APIMappingError.invalidPayload(
                "system info carries a partial OAuth configuration"
            )
        }

        return ServerInfo(
            version: payload.version,
            clientID: payload.client_id,
            tokenURL: tokenURL,
            authorizeURL: authorizeURL
        )
    }
}
