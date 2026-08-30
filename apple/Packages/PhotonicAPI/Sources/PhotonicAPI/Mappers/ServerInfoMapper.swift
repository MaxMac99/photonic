import PhotonicCore

/// Maps generated system-info payloads to Core models, validating OAuth
/// endpoints at the boundary (R9).
public enum ServerInfoMapper {
    public static func map(_ payload: Components.Schemas.InfoResponse) throws -> ServerInfo {
        guard
            let tokenURL = ServerURL(payload.token_url),
            let authorizeURL = ServerURL(payload.authorize_url)
        else {
            throw APIMappingError.invalidPayload("system info carries invalid OAuth URLs")
        }
        return ServerInfo(
            version: payload.version,
            clientID: payload.client_id,
            tokenURL: tokenURL,
            authorizeURL: authorizeURL
        )
    }
}
