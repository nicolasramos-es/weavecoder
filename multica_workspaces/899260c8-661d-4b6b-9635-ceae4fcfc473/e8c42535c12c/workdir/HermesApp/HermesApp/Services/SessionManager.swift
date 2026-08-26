// SessionManager.swift
// Manages chat sessions locally and syncs with the API Server.

import Foundation

// MARK: - SessionManager

/// Manages local session storage and syncs with the API Server.
@available(iOS 17.0, *)
@MainActor
public final class SessionManager: ObservableObject {
    
    @Published public var sessions: [ChatSession] = []
    @Published public var currentSession: ChatSession?
    @Published public var isLoading = false
    @Published public var errorMessage: String?
    @Published public var isOnline = false
    
    private let client: HermesClient
    private let userDefaultsKey = "hermes_sessions"
    private let userDefaults = UserDefaults.standard
    
    public init(client: HermesClient) {
        self.client = client
        self.loadSessionsLocally()
    }
    
    // MARK: - Public Methods
    
    /// Load all sessions from the server.
    public func loadSessions() async {
        isLoading = true
        errorMessage = nil
        
        do {
            let remoteSessions = try await client.listSessions()
            // Merge with local sessions (prefer remote)
            sessions = mergeSessions(remoteSessions, localSessions: sessions)
            saveSessionsLocally()
            isOnline = true
        } catch {
            errorMessage = "Failed to load sessions: \(error.localizedDescription)"
            isOnline = false
        }
        
        isLoading = false
    }
    
    /// Create a new session.
    public func createSession(title: String) async throws -> ChatSession {
        isLoading = true
        defer { isLoading = false }
        
        do {
            let session = try await client.createSession(title: title)
            sessions.insert(session, at: 0)
            saveSessionsLocally()
            return session
        } catch {
            throw SessionManagerError.createFailed(error.localizedDescription)
        }
    }
    
    /// Load a session's history.
    public func loadSessionHistory(sessionId: String) async throws -> [ChatMessage] {
        isLoading = true
        defer { isLoading = false }
        
        do {
            let messages = try await client.getSessionHistory(sessionId: sessionId)
            // Update the session in our list
            if let index = sessions.firstIndex(where: { $0.id == sessionId }) {
                var session = sessions[index]
                session.messages = messages
                session.updatedAt = Date()
                sessions[index] = session
                saveSessionsLocally()
            }
            return messages
        } catch {
            throw SessionManagerError.loadFailed(error.localizedDescription)
        }
    }
    
    /// Delete a session.
    public func deleteSession(sessionId: String) async throws {
        isLoading = true
        defer { isLoading = false }
        
        do {
            try await client.deleteSession(sessionId: sessionId)
            sessions.removeAll { $0.id == sessionId }
            if currentSession?.id == sessionId {
                currentSession = nil
            }
            saveSessionsLocally()
        } catch {
            throw SessionManagerError.deleteFailed(error.localizedDescription)
        }
    }
    
    /// Clear a session's messages.
    public func clearSession(sessionId: String) async throws {
        isLoading = true
        defer { isLoading = false }
        
        do {
            try await client.clearSession(sessionId: sessionId)
            if let index = sessions.firstIndex(where: { $0.id == sessionId }) {
                var session = sessions[index]
                session.messages = []
                session.updatedAt = Date()
                sessions[index] = session
                saveSessionsLocally()
            }
        } catch {
            throw SessionManagerError.clearFailed(error.localizedDescription)
        }
    }
    
    // MARK: - Local Storage
    
    private func saveSessionsLocally() {
        do {
            let data = try JSONEncoder().encode(sessions)
            userDefaults.set(data, forKey: userDefaultsKey)
        } catch {
            print("Failed to save sessions locally: \(error.localizedDescription)")
        }
    }
    
    private func loadSessionsLocally() {
        guard let data = userDefaults.data(forKey: userDefaultsKey) else {
            return
        }
        
        do {
            sessions = try JSONDecoder().decode([ChatSession].self, from: data)
        } catch {
            print("Failed to load sessions locally: \(error.localizedDescription)")
            sessions = []
        }
    }
    
    private func mergeSessions(_ remote: [ChatSession], localSessions: [ChatSession]) -> [ChatSession] {
        // Use remote sessions as source of truth, but keep local ones that don't exist remotely
        var merged = remote
        let remoteIds = Set(remote.map { $0.id })
        
        for local in localSessions where !remoteIds.contains(local.id) {
            merged.append(local)
        }
        
        // Sort by updatedAt descending
        return merged.sorted { $0.updatedAt > $1.updatedAt }
    }
}

// MARK: - Errors

enum SessionManagerError: Error, LocalizedError {
    case createFailed(String)
    case loadFailed(String)
    case deleteFailed(String)
    case clearFailed(String)
    
    var errorDescription: String? {
        switch self {
        case .createFailed(let msg): return "Failed to create session: \(msg)"
        case .loadFailed(let msg): return "Failed to load session: \(msg)"
        case .deleteFailed(let msg): return "Failed to delete session: \(msg)"
        case .clearFailed(let msg): return "Failed to clear session: \(msg)"
        }
    }
}
