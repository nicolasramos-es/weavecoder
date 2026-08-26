// Models.swift
// Data models for the Hermes iOS app.
// Shared between HermesClient and HermesApp targets.

import Foundation

// MARK: - Chat Message

/// Represents a single message in a chat conversation.
public struct ChatMessage: Identifiable, Codable, Hashable {
    public let id: String
    public let role: MessageRole
    public let content: String
    public let timestamp: Date
    
    public init(id: String = UUID().uuidString, role: MessageRole, content: String, timestamp: Date = Date()) {
        self.id = id
        self.role = role
        self.content = content
        self.timestamp = timestamp
    }
    
    public enum MessageRole: String, Codable, CaseIterable {
        case user
        case assistant
        case system
    }
    
    /// Request format for the API.
    public struct ChatMessageRequest: Codable, Hashable {
        public let role: String
        public let content: String
        
        public init(role: String, content: String) {
            self.role = role
            self.content = content
        }
    }
}

// MARK: - Session

/// Represents a chat session with the Hermes Bot.
public struct ChatSession: Identifiable, Codable, Hashable {
    public let id: String
    public var title: String
    public var messages: [ChatMessage]
    public var createdAt: Date
    public var updatedAt: Date
    
    public init(id: String = UUID().uuidString, title: String, messages: [ChatMessage] = [], createdAt: Date = Date(), updatedAt: Date = Date()) {
        self.id = id
        self.title = title
        self.messages = messages
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }
}

// MARK: - API Response Models

/// Response from the /health endpoint.
public struct HealthResponse: Codable {
    public let status: String
    public let platform: String
    public let version: String
}

/// Response from the /v1/models endpoint.
public struct ModelsResponse: Codable {
    public let data: [ModelInfo]
    
    public struct ModelInfo: Codable, Identifiable {
        public let id: String
        public let object: String
        public let created: UInt64
        public let ownedBy: String
    }
}

/// Request body for chat completions.
public struct ChatCompletionRequest: Codable {
    public let model: String
    public let messages: [ChatMessage.ChatMessageRequest]
    public let stream: Bool
    
    public init(model: String, messages: [ChatMessage.ChatMessageRequest], stream: Bool) {
        self.model = model
        self.messages = messages
        self.stream = stream
    }
}

/// Response from the /v1/chat/completions endpoint.
public struct ChatCompletionResponse: Codable {
    public let id: String
    public let object: String
    public let created: UInt64
    public let model: String
    public let choices: [Choice]
    public let usage: Usage?
    
    public struct Choice: Codable {
        public let index: Int
        public let message: AssistantMessage
        public let finishReason: String?
        
        public struct AssistantMessage: Codable {
            public let role: String
            public let content: String?
        }
    }
    
    public struct Usage: Codable {
        public let promptTokens: Int
        public let completionTokens: Int
        public let totalTokens: Int
        
        enum CodingKeys: String, CodingKey {
            case promptTokens = "prompt_tokens"
            case completionTokens = "completion_tokens"
            case totalTokens = "total_tokens"
        }
    }
}

/// Streaming response chunk from SSE.
public struct ChatCompletionChunk: Codable {
    public let id: String
    public let object: String
    public let created: UInt64
    public let model: String
    public let choices: [ChunkChoice]
    
    public struct ChunkChoice: Codable {
        public let index: Int
        public let delta: ChunkDelta
        public let finishReason: String?
        
        public struct ChunkDelta: Codable {
            public let role: String?
            public let content: String?
        }
    }
}

// MARK: - Error Models

/// Error response from the API.
public struct APIError: Error, Codable, LocalizedError {
    public let message: String
    public let type: String
    public let code: String
    
    public var errorDescription: String? {
        return message
    }
    
    enum CodingKeys: String, CodingKey {
        case message
        case type
        case code
    }
}

// MARK: - Command Models

/// Represents a slash command available in the app.
public struct SlashCommand: Identifiable, Codable, Hashable {
    public let id: String
    public let name: String
    public let description: String
    public let handler: String
    
    enum CodingKeys: String, CodingKey {
        case id = "id"
        case name
        case description
        case handler
    }
}

// MARK: - Status Model

/// Status information from the API Server.
public struct ServerStatus: Codable {
    public let status: String
    public let platform: String
    public let version: String
    public let uptime: String?
    public let activeSessions: Int?
    public let totalSessions: Int?
}

// MARK: - Notification Model

/// Push notification payload.
public struct PushNotification: Codable {
    public let id: String
    public let type: NotificationType
    public let title: String
    public let body: String
    public let timestamp: Date
    public let data: [String: AnyCodable]?
    
    public enum NotificationType: String, Codable {
        case messageReceived
        case sessionCompleted
        case error
        case system
    }
}

// MARK: - AnyCodable Helper

/// Simple AnyCodable for optional dictionary values.
public struct AnyCodable: Codable {
    public let value: Any
    
    public init(_ value: Any) {
        self.value = value
    }
    
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let bool = try? container.decode(Bool.self) {
            value = bool
        } else if let int = try? container.decode(Int.self) {
            value = int
        } else if let double = try? container.decode(Double.self) {
            value = double
        } else if let string = try? container.decode(String.self) {
            value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            value = array.map { $0.value }
        } else if let dict = try? container.decode([String: AnyCodable].self) {
            value = dict
        } else {
            value = NSNull()
        }
    }
    
    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch value {
        case let bool as Bool: try container.encode(bool)
        case let int as Int: try container.encode(int)
        case let double as Double: try container.encode(double)
        case let string as String: try container.encode(string)
        case let array as [Any]:
            let encodables = array.map { AnyCodable($0) }
            try container.encode(encodables)
        case let dict as [String: Any]:
            let encodableDict = dict.mapValues { AnyCodable($0) }
            try container.encode(encodableDict)
        case is NSNull:
            try container.encodeNil()
        default:
            throw EncodingError.invalidValue(value, EncodingError.Context(codingPath: encoder.codingPath, debugDescription: "Unsupported type"))
        }
    }
}
