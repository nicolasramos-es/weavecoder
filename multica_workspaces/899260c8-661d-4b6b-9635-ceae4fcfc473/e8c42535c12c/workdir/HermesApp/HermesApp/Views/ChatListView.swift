// ChatListView.swift
// Session list view with swipe-to-delete and session selection.

import SwiftUI

// MARK: - ChatListView

@available(iOS 17.0, *)
struct ChatListView: View {
    @EnvironmentObject var sessionManager: SessionManager
    @EnvironmentObject var chatViewModel: ChatViewModel
    @State private var showingNewSessionSheet = false
    @State private var newSessionTitle = ""
    @State private var showingServerStatus = false
    @State private var serverStatusText: String = ""
    
    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                if sessionManager.isLoading {
                    ProgressView("Loading sessions...")
                        .padding()
                } else if sessionManager.sessions.isEmpty {
                    emptyState
                } else {
                    sessionList
                }
            }
            .navigationTitle("Chats")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button {
                        checkServerStatus()
                    } label: {
                        Image(systemName: "server.rack")
                            .foregroundColor(.blue)
                    }
                    .help("Server Status")
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button {
                        showingNewSessionSheet = true
                    } label: {
                        Image(systemName: "square.and.pencil")
                            .font(.title3)
                    }
                }
            }
            .sheet(isPresented: $showingNewSessionSheet) {
                newSessionSheet
            }
            .alert("Server Status", isPresented: $showingServerStatus) {
                Button("OK") {}
            } message: {
                Text(serverStatusText)
            }
            .onAppear {
                Task {
                    await sessionManager.loadSessions()
                }
            }
        }
    }
    
    // MARK: - Empty State
    
    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "message.badge.fill")
                .font(.system(size: 60))
                .foregroundColor(.gray)
            
            Text("No chats yet")
                .font(.title2)
                .fontWeight(.semibold)
            
            Text("Start a new conversation with Hermes Bot")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
            
            Button {
                showingNewSessionSheet = true
            } label: {
                Label("Start New Chat", systemImage: "plus")
                    .font(.headline)
                    .foregroundColor(.white)
                    .padding(.horizontal, 24)
                    .padding(.vertical, 12)
                    .background(Color.blue)
                    .clipShape(RoundedRectangle(cornerRadius: 12))
            }
        }
    }
    
    // MARK: - Session List
    
    private var sessionList: some View {
        List {
            ForEach(sessionManager.sessions) { session in
                NavigationLink(destination: chatDestination(for: session)) {
                    sessionRow(for: session)
                }
                .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                    Button(role: .destructive) {
                        Task {
                            try? await sessionManager.deleteSession(session.id)
                        }
                    } label: {
                        Label("Delete", systemImage: "trash")
                    }
                    
                    Button {
                        Task {
                            try? await sessionManager.clearSession(session.id)
                        }
                    } label: {
                        Label("Clear", systemImage: "arrow.counterclockwise")
                    }
                    .tint(.orange)
                }
            }
        }
        .listStyle(.plain)
    }
    
    // MARK: - Session Row
    
    private func sessionRow(for session: ChatSession) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(session.title)
                .font(.headline)
            
            if let lastMessage = session.messages.last {
                Text(lastMessage.content)
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .lineLimit(2)
            }
            
            Text(formatRelativeTime(session.updatedAt))
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 4)
    }
    
    // MARK: - Chat Destination
    
    private func chatDestination(for session: ChatSession) -> some View {
        ChatView()
            .environmentObject(chatViewModel)
            .onAppear {
                chatViewModel.selectSession(session)
            }
    }
    
    // MARK: - New Session Sheet
    
    private var newSessionSheet: some View {
        NavigationStack {
            Form {
                Section("Session Title") {
                    TextField("Enter title", text: $newSessionTitle)
                }
                
                Section("Quick Actions") {
                    Button {
                        Task {
                            try? await sessionManager.createSession(title: newSessionTitle.isEmpty ? "New Chat" : newSessionTitle)
                            newSessionTitle = ""
                            showingNewSessionSheet = false
                        }
                    } label: {
                        Label("Create Session", systemImage: "plus.circle")
                    }
                }
            }
            .navigationTitle("New Chat")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        newSessionTitle = ""
                        showingNewSessionSheet = false
                    }
                }
            }
        }
    }
    
    // MARK: - Server Status
    
    private func checkServerStatus() {
        Task {
            do {
                let status = try await chatViewModel.client.checkHealth()
                serverStatusText = """
                Platform: \(status.platform)
                Version: \(status.version)
                Status: \(status.status)
                """
            } catch {
                serverStatusText = "Error: \(error.localizedDescription)"
            }
            showingServerStatus = true
        }
    }
    
    // MARK: - Helpers
    
    private func formatRelativeTime(_ date: Date) -> String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}

// MARK: - Preview

#if DEBUG
@available(iOS 17.0, *)
struct ChatListView_Previews: PreviewProvider {
    static var previews: some View {
        ChatListView()
            .environmentObject(SessionManager(client: HermesClient(config: .init(baseURL: URL(string: "http://localhost:8642")!, apiKey: "test"))))
            .environmentObject(ChatViewModel(client: HermesClient(config: .init(baseURL: URL(string: "http://localhost:8642")!, apiKey: "test"))))
    }
}
#endif
