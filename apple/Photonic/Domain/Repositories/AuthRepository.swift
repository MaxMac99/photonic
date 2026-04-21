//
//  AuthRepository.swift
//  Photonic
//
//  Domain Repository Protocol
//

import Foundation

protocol AuthRepository {
    func getUserAccount() async throws -> UserAccount
    func signInInteractive() async throws -> (access: AccessToken, refresh: RefreshToken)
    func signOut() async throws
}
