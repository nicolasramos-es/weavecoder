// HermesClient.swift
// HTTP client for the Hermes API Server (port 8642, OpenAI-compatible).

import Foundation

// MARK: - HermesClient

/// Client for communicating with the Hermes API Server.
@available(iOS 15.0, *)
public final class HermesClient {
    
    public enum ClientError: Error, LocalizedError {
        case invalidURL
        case authenticationFailed(message: String, type: String, code: String)
        case networkError(Error)
        case decodingError(Error)
        case emptyResponse
        case unknownError(String)
        
        public var errorDescription: String? {
            switch self {
            case .invalidURL:
                return "Invalid URL"
            case .authenticationFailed(let message, _, _):
                return "Authentication failed: \(message)"
            case .networkError(let error):
                return "Network error: \(error.localizedDescription)"
            case .decodingError(let error):
                return "Decoding error: \(error.localizedDescription)"
            case .emptyResponse:
                return "Empty response from server"
            case .unknownError(let message):
                return "Unknown error: \(message)"
            }
        }
    }
    
    public struct Config {
        public let baseURL: URL
        public let apiKey: String
        public let defaultModel: String
        public let timeout: TimeInterval
        
        public init(baseURL: URL, apiKey: String, defaultModel: String = "default", timeout: TimeInterval = 60.0) {
            self.baseURL = baseURL
            self.apiKey = apiKey
            self.defaultModel = defaultModel
            self.timeout = timeout
        }
    }
    
    private let config: Config
    private let session: URLSession
    
    public init(config: Config) {
        self.config = config
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = config.timeout
        configuration.timeoutIntervalForResource = config.timeout * 10
        self.session = URLSession(configuration: configuration)
    }
    
    // MARK: - Health Check
    
    /// Check if the API Server is reachable.
    public func checkHealth() async throws -> HealthResponse {
        let url = config.baseURL.appendingPathComponent("health")
        
        let (data, response) = try await session.data(from: url)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw ClientError.unknownError("Unexpected response")
        }
        
        do {
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            return try decoder.decode(HealthResponse.self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    // MARK: - Models
    
    /// List available models.
    public func listModels() async throws -> ModelsResponse {
        let url = config.baseURL.appendingPathComponent("v1/models")
        
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw ClientError.unknownError("Unexpected response")
        }
        
        if httpResponse.statusCode == 401 || httpResponse.statusCode == 403 {
            // Parse error response for auth failure details
            do {
                let errorResponse = try JSONDecoder().decode(APIError.self, from: data)
                throw ClientError.authenticationFailed(
                    message: errorResponse.message,
                    type: errorResponse.type,
                    code: errorResponse.code
                )
            } catch {
                // Fall through to generic error
            }
        }
        
        guard httpResponse.statusCode == 200 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
        
        do {
            return try JSONDecoder().decode(ModelsResponse.self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    // MARK: - Chat Completions
    
    /// Send a chat message and get a response (non-streaming).
    public func sendChatMessage(
        messages: [ChatMessage.ChatMessageRequest],
        model: String? = nil,
        stream: Bool = false
    ) async throws -> ChatCompletionResponse {
        
        let url = config.baseURL.appendingPathComponent("v1/chat/completions")
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let requestBody = ChatCompletionRequest(
            model: model ?? config.defaultModel,
            messages: messages,
            stream: stream
        )
        
        request.httpBody = try JSONEncoder().encode(requestBody)
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw ClientError.unknownError("Unexpected response")
        }
        
        if httpResponse.statusCode == 401 || httpResponse.statusCode == 403 {
            do {
                let errorResponse = try JSONDecoder().decode(APIError.self, from: data)
                throw ClientError.authenticationFailed(
                    message: errorResponse.message,
                    type: errorResponse.type,
                    code: errorResponse.code
                )
            } catch {
                throw ClientError.authenticationFailed(
                    message: "Authentication failed",
                    type: "auth_error",
                    code: "auth_failed"
                )
            }
        }
        
        guard httpResponse.statusCode == 200 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
        
        do {
            return try JSONDecoder().decode(ChatCompletionResponse.self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    // MARK: - Streaming Chat
    
    /// Send a chat message with streaming response (SSE).
    public func sendStreamingChat(
        messages: [ChatMessage.ChatMessageRequest],
        model: String? = nil
    ) -> AsyncThrowingStream<ChatCompletionChunk, Error> {
        
        return AsyncThrowingStream { continuation in
            Task {
                do {
                    let url = config.baseURL.appendingPathComponent("v1/chat/completions")
                    
                    var request = URLRequest(url: url)
                    request.httpMethod = "POST"
                    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
                    request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
                    request.setValue("text/event-stream", forHTTPHeaderField: "Accept")
                    
                    let requestBody = ChatCompletionRequest(
                        model: model ?? config.defaultModel,
                        messages: messages,
                        stream: true
                    )
                    
                    request.httpBody = try JSONEncoder().encode(requestBody)
                    
                    let (dataStream, response) = try await session.bytes(for: request)
                    
                    guard let httpResponse = response as? HTTPURLResponse,
                          httpResponse.statusCode == 200 else {
                        continuation.finish(throwing: ClientError.unknownError("HTTP \(httpResponse.statusCode)"))
                        return
                    }
                    
                    // Parse SSE stream
                    var buffer = ""
                    for try await line in dataStream.lines {
                        if line.isEmpty {
                            // Empty line signals end of SSE event
                            if !buffer.isEmpty {
                                try await self.processSSEEvent(buffer, continuation: continuation)
                                buffer = ""
                            }
                        } else {
                            buffer += line + "\n"
                        }
                    }
                    
                    // Process any remaining buffer
                    if !buffer.isEmpty {
                        try await self.processSSEEvent(buffer, continuation: continuation)
                    }
                    
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }
    
    private func processSSEEvent(_ buffer: String, continuation: AsyncThrowingStream<ChatCompletionChunk, Error>.Continuation) async {
        guard buffer.hasPrefix("data: ") else { return }
        
        let dataString = String(buffer.dropFirst(6).dropLast()) // Remove "data: " and trailing newline
        guard !dataString.hasPrefix("[DONE]") else {
            return
        }
        
        do {
            let chunk = try JSONDecoder().decode(ChatCompletionChunk.self, from: dataString.data(using: .utf8)!)
            continuation.yield(chunk)
        } catch {
            continuation.finish(throwing: ClientError.decodingError(error))
        }
    }
    
    // MARK: - Sessions
    
    /// List all sessions.
    public func listSessions() async throws -> [ChatSession] {
        let url = config.baseURL.appendingPathComponent("api/sessions/list")
        
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
        
        do {
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .iso8601
            return try decoder.decode([ChatSession].self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    /// Create a new session.
    public func createSession(title: String) async throws -> ChatSession {
        let url = config.baseURL.appendingPathComponent("api/sessions/new")
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let body: [String: Any] = ["title": title]
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 || httpResponse.statusCode == 201 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
        
        do {
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .iso8601
            return try decoder.decode(ChatSession.self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    /// Get session history.
    public func getSessionHistory(sessionId: String) async throws -> [ChatMessage] {
        let url = config.baseURL.appendingPathComponent("api/sessions/history")
        
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
        
        do {
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .iso8601
            return try decoder.decode([ChatMessage].self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    /// Clear a session.
    public func clearSession(sessionId: String) async throws {
        let url = config.baseURL.appendingPathComponent("api/sessions/clear")
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 || httpResponse.statusCode == 204 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
    }
    
    /// Delete a session.
    public func deleteSession(sessionId: String) async throws {
        let url = config.baseURL.appendingPathComponent("api/sessions/delete")
        
        var request = URLRequest(url: url)
        request.httpMethod = "DELETE"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 || httpResponse.statusCode == 204 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
    }
    
    // MARK: - Commands
    
    /// List available slash commands.
    public func listCommands() async throws -> [SlashCommand] {
        let url = config.baseURL.appendingPathComponent("api/commands")
        
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
        
        do {
            return try JSONDecoder().decode([SlashCommand].self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    // MARK: - Status
    
    /// Get server status.
    public func getServerStatus() async throws -> ServerStatus {
        let url = config.baseURL.appendingPathComponent("api/status")
        
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
        
        do {
            return try JSONDecoder().decode(ServerStatus.self, from: data)
        } catch {
            throw ClientError.decodingError(error)
        }
    }
    
    // MARK: - Notifications
    
    /// Register for push notifications.
    public func registerForNotifications(deviceToken: String) async throws {
        let url = config.baseURL.appendingPathComponent("api/notifications/register")
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        
        let body: [String: String] = ["deviceToken": deviceToken]
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
        
        let (_, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 || httpResponse.statusCode == 201 else {
            throw ClientError.unknownError("HTTP \(httpResponse.statusCode)")
        }
    }
}
