import { useState, useEffect, useRef } from "react";
import { Button } from "./ui/button";
import { Textarea } from "./ui/textarea";
import { Message, CreateMessageInput, Conversation, Project } from "../types";
import { Send, Bot, User } from "lucide-react";

interface ChatViewProps {
  project: Project;
  conversation: Conversation | null;
  messages: Message[];
  onSendMessage: (input: CreateMessageInput) => void;
  onCreateConversation: (projectId: string) => void;
  loading: boolean;
}

export function ChatView({
  project,
  conversation,
  messages,
  onSendMessage,
  onCreateConversation,
  loading,
}: ChatViewProps) {
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  useEffect(() => {
    if (project && !conversation && !loading) {
      onCreateConversation(project.id);
    }
  }, [project, conversation, onCreateConversation, loading]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || !conversation || loading) return;

    onSendMessage({
      conversation_id: conversation.id,
      role: "user",
      content: input.trim(),
    });
    setInput("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  return (
    <div className="flex-1 flex flex-col h-screen">
      <div className="border-b p-4">
        <h2 className="text-xl font-semibold">{project.name}</h2>
        <p className="text-sm text-gray-500">{project.description}</p>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            <div className="text-center">
              <Bot className="w-16 h-16 mx-auto mb-4 opacity-50" />
              <p className="text-lg font-medium mb-2">Start a conversation</p>
              <p className="text-sm">
                Describe your product idea and I'll help you create a detailed
                specification
              </p>
            </div>
          </div>
        ) : (
          messages.map((message) => (
            <div
              key={message.id}
              className={`flex gap-3 ${
                message.role === "user" ? "justify-end" : "justify-start"
              }`}
            >
              {message.role === "assistant" && (
                <div className="w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center flex-shrink-0">
                  <Bot className="w-5 h-5 text-white" />
                </div>
              )}
              <div
                className={`rounded-lg p-3 max-w-[70%] ${
                  message.role === "user"
                    ? "bg-blue-600 text-white"
                    : "bg-gray-100"
                }`}
              >
                <p className="text-sm whitespace-pre-wrap">{message.content}</p>
              </div>
              {message.role === "user" && (
                <div className="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center flex-shrink-0">
                  <User className="w-5 h-5 text-gray-900" />
                </div>
              )}
            </div>
          ))
        )}
        {loading && (
          <div className="flex gap-3">
            <div className="w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center flex-shrink-0">
              <Bot className="w-5 h-5 text-white" />
            </div>
            <div className="rounded-lg p-3 bg-gray-100">
              <div className="flex gap-1">
                <div className="w-2 h-2 rounded-full bg-gray-500 animate-bounce" />
                <div className="w-2 h-2 rounded-full bg-gray-500 animate-bounce delay-100" />
                <div className="w-2 h-2 rounded-full bg-gray-500 animate-bounce delay-200" />
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="border-t p-4">
        <form onSubmit={handleSubmit} className="flex gap-2">
          <Textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type your message... (Shift+Enter for new line)"
            className="flex-1 resize-none"
            rows={3}
            disabled={loading || !conversation}
          />
          <Button
            type="submit"
            size="icon"
            disabled={!input.trim() || loading || !conversation}
          >
            <Send className="w-4 h-4" />
          </Button>
        </form>
      </div>
    </div>
  );
}
