use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone)] 
struct Todo {
    id: usize,
    title: String,
    completed: bool,
}

impl Clone for Todo {
    fn clone(&self) -> Self {
        // Create a new Todo struct with copied values
        Todo {
            id: self.id.clone(), // Clone the usize value
            title: self.title.clone(), // Clone the String value (heap-allocated)
            completed: self.completed, // Copy the bool value (stack-allocated)
        }
    }
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Deserialize)]
#[derive(Debug)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

const TODOS_FILE: &str = "todos.json";

fn read_todos_from_file() -> Vec<Todo> {
    if let Ok(mut file) = fs::File::open(TODOS_FILE) {
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap_or_default();
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn write_todos_to_file(todos: &[Todo]) {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(TODOS_FILE)
        .expect("Failed to open file");

    let todos_json = serde_json::to_string_pretty(todos).expect("Failed to serialize todos");
    file.write_all(todos_json.as_bytes()).expect("Failed to write to file");
}

fn get_next_id(todos: &[Todo]) -> usize {
    todos.iter().map(|todo| todo.id).max().unwrap_or(0) + 1
}

async fn get_todos() -> impl Responder {
    let todos = read_todos_from_file();
    HttpResponse::Ok().json(todos)
}

async fn create_todo(todo: web::Json<CreateTodo>) -> impl Responder {
    let mut todos = read_todos_from_file();
    let new_id = get_next_id(&todos);
    let new_todo = Todo {
        id: new_id,
        title: todo.title.clone(),
        completed: false,
    };
    todos.push(new_todo.clone());
    write_todos_to_file(&todos);
    HttpResponse::Created().json(new_todo)
}

async fn update_todo(todo_id: web::Path<usize>, updated_todo: web::Json<UpdateTodo>) -> impl Responder {
    let mut todos = read_todos_from_file();
    if let Some(todo) = todos.iter_mut().find(|t| t.id == *todo_id) {
        if let Some(title) = &updated_todo.title {
            todo.title = title.clone();
        }
        if let Some(completed) = updated_todo.completed {
            todo.completed = completed;
        }
        write_todos_to_file(&todos);
        HttpResponse::Ok().body(format!("Updated todo with ID {}: {:?}", todo_id, updated_todo))
    } else {
        HttpResponse::NotFound().body("Todo not found")
    }
}

async fn delete_todo(todo_id: web::Path<usize>) -> impl Responder {
    let mut todos = read_todos_from_file();
    if let Some(index) = todos.iter().position(|t| t.id == *todo_id) {
        todos.remove(index);
        write_todos_to_file(&todos);
        HttpResponse::Ok().body(format!("Deleted todo with ID {}", todo_id))
    } else {
        HttpResponse::NotFound().body("Todo not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/todos", web::get().to(get_todos))
            .route("/todos", web::post().to(create_todo))
            .route("/todos/{id}", web::put().to(update_todo))
            .route("/todos/{id}", web::delete().to(delete_todo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
