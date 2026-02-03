import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "./components/Sidebar";
import { ChatView } from "./components/ChatView";
import {
  Project,
  CreateProjectInput,
  Conversation,
  Message,
  CreateMessageInput,
} from "./types";

function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadProjects();
  }, []);

  useEffect(() => {
    if (selectedProject) {
      loadOrCreateConversation(selectedProject.id);
    }
  }, [selectedProject]);

  const loadProjects = async () => {
    try {
      const projectList = await invoke<Project[]>("get_projects");
      setProjects(projectList);
    } catch (error) {
      console.error("Failed to load projects:", error);
    }
  };

  const loadOrCreateConversation = async (projectId: string) => {
    try {
      const newConversation = await invoke<Conversation>(
        "create_conversation",
        { projectId }
      );
      setConversation(newConversation);
      loadMessages(newConversation.id);
    } catch (error) {
      console.error("Failed to load conversation:", error);
    }
  };

  const loadMessages = async (conversationId: string) => {
    try {
      const messageList = await invoke<Message[]>(
        "get_conversation_messages",
        { conversationId }
      );
      setMessages(messageList);
    } catch (error) {
      console.error("Failed to load messages:", error);
    }
  };

  const handleCreateProject = async (input: CreateProjectInput) => {
    try {
      const newProject = await invoke<Project>("create_project", { input });
      setProjects([newProject, ...projects]);
      setSelectedProject(newProject);
    } catch (error) {
      console.error("Failed to create project:", error);
    }
  };

  const handleSendMessage = async (input: CreateMessageInput) => {
    if (!conversation) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      conversation_id: conversation.id,
      role: "user",
      content: input.content,
      created_at: new Date().toISOString(),
    };

    setMessages([...messages, userMessage]);
    setLoading(true);

    try {
      const assistantMessage = await invoke<Message>("send_message", {
        input,
      });
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      console.error("Failed to send message:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateConversation = async (projectId: string) => {
    await loadOrCreateConversation(projectId);
  };

  if (!selectedProject) {
    return (
      <div className="flex h-screen bg-white text-gray-900">
        <Sidebar
          projects={projects}
          selectedProject={selectedProject}
          onSelectProject={setSelectedProject}
          onCreateProject={handleCreateProject}
        />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center space-y-4">
            <h1 className="text-4xl font-bold">SpecMaker</h1>
            <p className="text-gray-500">
              AI-Powered Product Specification Tool
            </p>
            <p className="text-sm text-gray-500">
              Create a new project or select an existing one to get started
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-white text-gray-900">
      <Sidebar
        projects={projects}
        selectedProject={selectedProject}
        onSelectProject={setSelectedProject}
        onCreateProject={handleCreateProject}
      />
      <ChatView
        project={selectedProject}
        conversation={conversation}
        messages={messages}
        onSendMessage={handleSendMessage}
        onCreateConversation={handleCreateConversation}
        loading={loading}
      />
    </div>
  );
}

export default App;
