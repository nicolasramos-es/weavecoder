// HermesClientTests.swift
// Unit tests for the HermesClient HTTP client.

import XCTest
@testable import HermesClient

final class HermesClientTests: XCTestCase {
    
    var client: HermesClient!
    var mockServer: HTTPMockServer!
    
    override func setUp() {
        super.setUp()
        mockServer = HTTPMockServer()
        let config = HermesClient.Config(
            baseURL: URL(string: "http://localhost:8642")!,
            apiKey: "test-key",
            defaultModel: "test-model"
        )
        client = HermesClient(config: config)
    }
    
    override func tearDown() {
        mockServer = nil
        client = nil
        super.tearDown()
    }
    
    // MARK: - Config Tests
    
    func testConfigInitialization() {
        let config = HermesClient.Config(
            baseURL: URL(string: "http://localhost:8642")!,
            apiKey: "test-key",
            defaultModel: "test-model",
            timeout: 30.0
        )
        
        XCTAssertEqual(config.baseURL.absoluteString, "http://localhost:8642")
        XCTAssertEqual(config.apiKey, "test-key")
        XCTAssertEqual(config.defaultModel, "test-model")
        XCTAssertEqual(config.timeout, 30.0)
    }
    
    // MARK: - Model Tests
    
    func testChatMessageInitialization() {
        let message = ChatMessage(
            id: "test-id",
            role: .user,
            content: "Hello",
            timestamp: Date()
        )
        
        XCTAssertEqual(message.id, "test-id")
        XCTAssertEqual(message.role, .user)
        XCTAssertEqual(message.content, "Hello")
    }
    
    func testChatMessageRoleEnum() {
        let roles = ChatMessage.MessageRole.allCases
        XCTAssertEqual(roles.count, 3)
        XCTAssertTrue(roles.contains(.user))
        XCTAssertTrue(roles.contains(.assistant))
        XCTAssertTrue(roles.contains(.system))
    }
    
    func testChatMessageHashable() {
        let msg1 = ChatMessage(id: "1", role: .user, content: "test")
        let msg2 = ChatMessage(id: "1", role: .user, content: "test")
        let msg3 = ChatMessage(id: "2", role: .user, content: "test")
        
        XCTAssertEqual(msg1.hashValue, msg2.hashValue)
        XCTAssertNotEqual(msg1.hashValue, msg3.hashValue)
    }
    
    // MARK: - Session Tests
    
    func testSessionInitialization() {
        let session = ChatSession(
            id: "session-1",
            title: "Test Session",
            messages: [],
            createdAt: Date(),
            updatedAt: Date()
        )
        
        XCTAssertEqual(session.id, "session-1")
        XCTAssertEqual(session.title, "Test Session")
        XCTAssertTrue(session.messages.isEmpty)
    }
    
    func testSessionIdentifiable() {
        let session1 = ChatSession(id: "1", title: "Session 1")
        let session2 = ChatSession(id: "2", title: "Session 2")
        
        XCTAssertNotEqual(session1.id, session2.id)
    }
    
    // MARK: - API Error Tests
    
    func testAPIErrorLocalizedDescription() {
        let error = APIError(
            message: "Test error",
            type: "test_type",
            code: "test_code"
        )
        
        XCTAssertEqual(error.errorDescription, "Test error")
    }
    
    func testClientErrorAuthenticationFailed() {
        let error = HermesClient.ClientError.authenticationFailed(
            message: "Invalid key",
            type: "auth_error",
            code: "auth_failed"
        )
        
        XCTAssertTrue(error.errorDescription?.contains("Invalid key") ?? false)
    }
    
    func testClientErrorNetworkError() {
        let nsError = NSError(domain: "test", code: 500, userInfo: nil)
        let error = HermesClient.ClientError.networkError(nsError)
        
        XCTAssertTrue(error.errorDescription?.contains("Network error") ?? false)
    }
    
    // MARK: - Request Body Tests
    
    func testChatCompletionRequestEncoding() {
        let request = ChatCompletionRequest(
            model: "test-model",
            messages: [
                ChatMessage.ChatMessageRequest(role: "user", content: "Hello")
            ],
            stream: false
        )
        
        let encoder = JSONEncoder()
        XCTAssertNoThrow(try encoder.encode(request))
    }
    
    // MARK: - Health Response Tests
    
    func testHealthResponseDecoding() {
        let json = """
        {
            "status": "ok",
            "platform": "hermes-agent",
            "version": "0.20.5"
        }
        """
        
        let data = json.data(using: .utf8)!
        let decoder = JSONDecoder()
        
        XCTAssertNoThrow(try decoder.decode(HealthResponse.self, from: data))
    }
    
    // MARK: - Models Response Tests
    
    func testModelsResponseDecoding() {
        let json = """
        {
            "data": [
                {
                    "id": "model-1",
                    "object": "model",
                    "created": 1234567890,
                    "ownedBy": "owner"
                }
            ]
        }
        """
        
        let data = json.data(using: .utf8)!
        let decoder = JSONDecoder()
        
        let response = try? decoder.decode(ModelsResponse.self, from: data)
        XCTAssertNotNil(response)
        XCTAssertEqual(response?.data.count, 1)
        XCTAssertEqual(response?.data.first?.id, "model-1")
    }
    
    // MARK: - Chat Completion Response Tests
    
    func testChatCompletionResponseDecoding() {
        let json = """
        {
            "id": "chat-1",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "test-model",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello!"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }
        """
        
        let data = json.data(using: .utf8)!
        let decoder = JSONDecoder()
        
        let response = try? decoder.decode(ChatCompletionResponse.self, from: data)
        XCTAssertNotNil(response)
        XCTAssertEqual(response?.id, "chat-1")
        XCTAssertEqual(response?.choices.count, 1)
        XCTAssertEqual(response?.choices.first?.message.content, "Hello!")
        XCTAssertEqual(response?.usage?.promptTokens, 10)
    }
    
    // MARK: - Slash Command Tests
    
    func testSlashCommandInitialization() {
        let command = SlashCommand(
            id: "cmd-1",
            name: "/new",
            description: "Create new session",
            handler: "new_session"
        )
        
        XCTAssertEqual(command.id, "cmd-1")
        XCTAssertEqual(command.name, "/new")
        XCTAssertEqual(command.description, "Create new session")
    }
    
    func testSlashCommandHashable() {
        let cmd1 = SlashCommand(id: "1", name: "/test", description: "Test", handler: "test")
        let cmd2 = SlashCommand(id: "1", name: "/test", description: "Test", handler: "test")
        
        XCTAssertEqual(cmd1.hashValue, cmd2.hashValue)
    }
    
    // MARK: - Server Status Tests
    
    func testServerStatusDecoding() {
        let json = """
        {
            "status": "ok",
            "platform": "hermes-agent",
            "version": "0.20.5",
            "uptime": "1h",
            "activeSessions": 5,
            "totalSessions": 100
        }
        """
        
        let data = json.data(using: .utf8)!
        let decoder = JSONDecoder()
        
        let status = try? decoder.decode(ServerStatus.self, from: data)
        XCTAssertNotNil(status)
        XCTAssertEqual(status?.status, "ok")
        XCTAssertEqual(status?.activeSessions, 5)
    }
    
    // MARK: - AnyCodable Tests
    
    func testAnyCodableBool() {
        let any = AnyCodable(true)
        XCTAssertNoThrow(try JSONEncoder().encode(any))
    }
    
    func testAnyCodableInt() {
        let any = AnyCodable(42)
        XCTAssertNoThrow(try JSONEncoder().encode(any))
    }
    
    func testAnyCodableString() {
        let any = AnyCodable("test")
        XCTAssertNoThrow(try JSONEncoder().encode(any))
    }
    
    func testAnyCodableArray() {
        let any = AnyCodable([1, 2, 3])
        XCTAssertNoThrow(try JSONEncoder().encode(any))
    }
    
    func testAnyCodableDictionary() {
        let any = AnyCodable(["key": "value"])
        XCTAssertNoThrow(try JSONEncoder().encode(any))
    }
    
    // MARK: - SessionManager Tests
    
    func testSessionManagerInitialState() {
        let client = HermesClient(config: .init(
            baseURL: URL(string: "http://localhost:8642")!,
            apiKey: "test"
        ))
        let manager = SessionManager(client: client)
        
        XCTAssertTrue(manager.sessions.isEmpty)
        XCTAssertNil(manager.currentSession)
        XCTAssertFalse(manager.isLoading)
        XCTAssertNil(manager.errorMessage)
    }
    
    func testSessionManagerErrorStates() {
        XCTAssertEqual(SessionManagerError.createFailed("test").errorDescription, "Failed to create session: test")
        XCTAssertEqual(SessionManagerError.loadFailed("test").errorDescription, "Failed to load session: test")
        XCTAssertEqual(SessionManagerError.deleteFailed("test").errorDescription, "Failed to delete session: test")
        XCTAssertEqual(SessionManagerError.clearFailed("test").errorDescription, "Failed to clear session: test")
    }
}

// MARK: - HTTP Mock Server (for testing without real server)

final class HTTPMockServer {
    // Placeholder for future mock server implementation
    // Would use OHHTTPStubs or similar for unit testing HTTP calls
}
