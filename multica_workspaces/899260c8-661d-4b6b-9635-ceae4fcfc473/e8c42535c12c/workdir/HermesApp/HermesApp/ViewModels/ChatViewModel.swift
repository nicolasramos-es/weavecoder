// ChatViewModel.swift
// ViewModel for the chat interface — manages message flow and session state.

import Foundation

// MARK: - ChatViewModel

@available(iOS 17.0, *)
@MainActor
public final class ChatViewModel: ObservableObject {
    
    @Published public var messages: [ChatMessage] = []
    @Published public var currentSession: ChatSession?
    @Published public var isSending = false
    @Published public var showError = false
    @Published public var errorMessage = ""
    @Published public var availableCommands: [SlashCommand] = []
    
    public let client: HermesClient
    
    private var currentSessionId: String?
    
    public init(client: HermesClient) {
        self.client = client
    }
    
    // MARK: - Public Methods
    
    /// Load messages for the current session.
    public func loadMessages() {
        guard let sessionId = currentSessionId else { return }
        
        Task {
            do {
                let history = try await client.getSessionHistory(sessionId: sessionId)
                messages = history
            } catch {
                errorMessage = "Failed to load messages: \(error.localizedDescription)"
                showError = true
            }
        }
    }
    
    /// Send a message to the bot.
    public func sendMessage(_ text: String) {
        guard let sessionId = currentSessionId else {
            errorMessage = "No active session"
            showError = true
            return
        }
        
        isSending = true
        
        // Add user message locally
        let userMessage = ChatMessage(role: .user, content: text)
        messages.append(userMessage)
        
        Task {
            do {
                // Build API messages
                let apiMessages = messages.map { msg in
                    ChatMessage.ChatMessageRequest(role: msg.role.rawValue, content: msg.content)
                }
                
                // Send to API (non-streaming for simplicity)
                let response = try await client.sendChatMessage(
                    messages: apiMessages,
                    model: nil,
                    stream: false
                )
                
                // Extract assistant response
                if let assistantContent = response.choices.first?.message.content {
                    let assistantMessage = ChatMessage(
                        role: .assistant,
                        content: assistantContent
                    )
                    messages.append(assistantMessage)
                }
                
                // Update session if needed
                if let session = currentSession {
                    var updatedSession = session
                    updatedSession.messages = messages
                    updatedSession.updatedAt = Date()
                    currentSession = updatedSession
                }
                
            } catch {
                errorMessage = "Failed to send message: \(error.localizedDescription)"
                showError = true
                // Remove the user message that failed
                if messages.last?.role == .user {
                    messages.removeLast()
                }
            }
            
            isSending = false
        }
    }
    
    /// Select a session to chat with.
    public func selectSession(_ session: ChatSession) {
        currentSession = session
        currentSessionId = session.id
        messages = session.messages
    }
    
    /// Create a new session.
    public func createNewSession() {
        Task {
            do {
                let title = "New Chat \(Date().formatted(date: .abbreviated, time: .shortened))"
                let session = try await client.createSession(title: title)
                currentSession = session
                currentSessionId = session.id
                messages = []
            } catch {
                errorMessage = "Failed to create session: \(error.localizedDescription)"
                showError = true
            }
        }
    }
    
    /// Clear the current session's messages.
    public func clearCurrentSession() {
        guard let sessionId = currentSessionId else { return }
        
        Task {
            do {
                try await client.clearSession(sessionId: sessionId)
                messages = []
                if let session = currentSession {
                    var updatedSession = session
                    updatedSession.messages = []
                    updatedSession.updatedAt = Date()
                    currentSession = updatedSession
                }
            } catch {
                errorMessage = "Failed to clear session: \(error.localizedDescription)"
                showError = true
            }
        }
    }
    
    /// Delete the current session.
    public func deleteCurrentSession() {
        guard let sessionId = currentSessionId else { return }
        
        Task {
            do {
                try await client.deleteSession(sessionId: sessionId)
                currentSession = nil
                currentSessionId = nil
                messages = []
            } catch {
                errorMessage = "Failed to delete session: \(error.localizedDescription)"
                showError = true
            }
        }
    }
    
    /// Check server status.
    public func checkServerStatus() {
        Task {
            do {
                let status = try await client.checkHealth()
                print("Server: \(status.platform) v\(status.version)")
            } catch {
                errorMessage = "Server check failed: \(error.localizedDescription)"
                showError = true
            }
        }
    }
    
    /// Load available slash commands.
    public func loadCommands() {
        Task {
            do {
                let commands = try await client.listCommands()
                availableCommands = commands
            } catch {
                // Commands are optional, log but don't error
                print("Failed to load commands: \(error.localizedDescription)")
                availableCommands = []
            }
        }
    }
}
