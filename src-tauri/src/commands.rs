use crate::database::{models::*, Database};
use crate::services::ollama::{ChatMessage, OllamaService};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn create_project(
    db: State<'_, Database>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO projects (id, name, description, industry, target_audience, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'ideation', ?6, ?6)",
        (
            &id,
            &input.name,
            &input.description,
            &input.industry,
            &input.target_audience,
            &now,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(Project {
        id,
        name: input.name,
        description: input.description,
        industry: input.industry,
        target_audience: input.target_audience,
        status: "ideation".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn get_projects(db: State<'_, Database>) -> Result<Vec<Project>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, description, industry, target_audience, status, created_at, updated_at FROM projects ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                industry: row.get(3)?,
                target_audience: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(projects)
}

#[tauri::command]
pub async fn get_project(db: State<'_, Database>, project_id: String) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let project = conn
        .query_row(
            "SELECT id, name, description, industry, target_audience, status, created_at, updated_at FROM projects WHERE id = ?1",
            [&project_id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    industry: row.get(3)?,
                    target_audience: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(project)
}

#[tauri::command]
pub async fn delete_project(db: State<'_, Database>, project_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM projects WHERE id = ?1", [&project_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn create_conversation(
    db: State<'_, Database>,
    project_id: String,
) -> Result<Conversation, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO conversations (id, project_id, phase, created_at) VALUES (?1, ?2, 'initial_analysis', ?3)",
        (&id, &project_id, &now),
    )
    .map_err(|e| e.to_string())?;

    Ok(Conversation {
        id,
        project_id,
        phase: "initial_analysis".to_string(),
        created_at: now,
    })
}

#[tauri::command]
pub async fn get_conversation_messages(
    db: State<'_, Database>,
    conversation_id: String,
) -> Result<Vec<Message>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, conversation_id, role, content, metadata, created_at FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;

    let messages = stmt
        .query_map([&conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                metadata: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(messages)
}

#[tauri::command]
pub async fn send_message(
    db: State<'_, Database>,
    ollama: State<'_, OllamaService>,
    input: CreateMessageInput,
) -> Result<Message, String> {
    let user_msg_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &user_msg_id,
                &input.conversation_id,
                &input.role,
                &input.content,
                &input.metadata,
                &now,
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    let messages = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, conversation_id, role, content, metadata, created_at FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;

        let messages = stmt
            .query_map([&input.conversation_id], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    metadata: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        messages
    };

    let ollama_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    let response_content = ollama.chat(ollama_messages).await?;

    let assistant_msg_id = Uuid::new_v4().to_string();
    let response_time = chrono::Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, metadata, created_at)
             VALUES (?1, ?2, 'assistant', ?3, NULL, ?4)",
            (
                &assistant_msg_id,
                &input.conversation_id,
                &response_content,
                &response_time,
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(Message {
        id: assistant_msg_id,
        conversation_id: input.conversation_id,
        role: "assistant".to_string(),
        content: response_content,
        metadata: None,
        created_at: response_time,
    })
}

#[tauri::command]
pub async fn check_ollama_connection(ollama: State<'_, OllamaService>) -> Result<bool, String> {
    ollama.check_connection().await
}
