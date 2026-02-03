export interface Project {
  id: string;
  name: string;
  description: string;
  industry?: string;
  target_audience?: string;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectInput {
  name: string;
  description: string;
  industry?: string;
  target_audience?: string;
}

export interface Conversation {
  id: string;
  project_id: string;
  phase: string;
  created_at: string;
}

export interface Message {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  metadata?: string;
  created_at: string;
}

export interface CreateMessageInput {
  conversation_id: string;
  role: string;
  content: string;
  metadata?: string;
}
