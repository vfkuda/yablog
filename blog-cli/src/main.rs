use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

use blog_client::{BlogClient, Transport};

const HTTP_DEFAULT_ADDR: &str = "http://localhost:8080";
const GRPC_DEFAULT_ADDR: &str = "localhost:50051";
const TOKEN_FILE: &str = ".blog_token";
const DEFAULT_LIST_LIMIT: usize = 10;
const DEFAULT_LIST_OFFSET: usize = 0;

#[derive(Debug, Parser)]
#[command(name = "blog-cli", version, about = "CLI client for YaBlog backend")]
struct Cli {
    #[arg(long, global = true)]
    grpc: bool,

    #[arg(long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Get {
        #[arg(long)]
        id: u64,
    },
    Update {
        #[arg(long)]
        id: u64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    Delete {
        #[arg(long)]
        id: u64,
    },
    Logout,
    List {
        #[arg(long, default_value_t = DEFAULT_LIST_LIMIT)]
        limit: usize,
        #[arg(long, default_value_t = DEFAULT_LIST_OFFSET)]
        offset: usize,
    },
}

#[tokio::main]
async fn main() {
    // dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let transport = if cli.grpc {
        let server = cli.server.unwrap_or_else(|| GRPC_DEFAULT_ADDR.to_string());
        println!("gRPC transport is used. Server is {server}");
        Transport::Grpc(server)
    } else {
        let server =
            normalize_http_server(cli.server.unwrap_or_else(|| HTTP_DEFAULT_ADDR.to_string()));
        println!("HTTP transport is used. Server is {server}");
        Transport::Http(server)
    };

    let mut client = match BlogClient::new(transport).await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("failed to initialize client: {err}");
            std::process::exit(1);
        }
    };

    if let Some(token) = load_saved_token(TOKEN_FILE) {
        client.set_token(token);
    }

    let result = run_command(&mut client, cli.command).await;
    if let Err(err) = result {
        eprintln!("command failed: {err}");
        std::process::exit(1);
    }
}

fn normalize_http_server(server: String) -> String {
    // пользователь часто передает просто host:port, добавляем схему молча
    if server.starts_with("http://") || server.starts_with("https://") {
        server
    } else {
        format!("http://{server}")
    }
}

fn load_saved_token(path: &str) -> Option<String> {
    // пустой файл токена считаем отсутствием сессии, а не ошибкой
    if !Path::new(path).exists() {
        return None;
    }

    let token = fs::read_to_string(path).ok()?;
    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn save_token(path: &str, token: &str) -> std::io::Result<()> {
    fs::write(path, token)
}

async fn run_command(
    client: &mut BlogClient,
    command: Commands,
) -> Result<(), Box<dyn std::error::Error>> {
    // CLI тонкий, вся сеть и auth-логика остаются в blog-client.
    match command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let auth = client.register(username, email, password).await?;
            save_token(TOKEN_FILE, &auth.token)?;
            println!("register ok");
            println!("user_id: {}", auth.user.id);
            println!("username: {}", auth.user.username);
            println!("token saved to {TOKEN_FILE}");
        }
        Commands::Login { username, password } => {
            let auth = client.login(username, password).await?;
            save_token(TOKEN_FILE, &auth.token)?;
            println!("login ok");
            println!("user_id: {}", auth.user.id);
            println!("username: {}", auth.user.username);
            println!("token saved to {TOKEN_FILE}");
        }
        Commands::Create { title, content } => {
            let post = client.create_post(title, content).await?;
            print_post(&post);
        }
        Commands::Get { id } => {
            let post = client.get_post(id as i64).await?;
            print_post(&post);
        }
        Commands::Update { id, title, content } => {
            let current = client.get_post(id as i64).await?;
            let new_title = title.unwrap_or(current.title);
            let new_content = content.unwrap_or(current.content);
            let post = client
                .update_post(id as i64, new_title, new_content)
                .await?;
            print_post(&post);
        }
        Commands::Delete { id } => {
            client.delete_post(id as i64).await?;
            println!("post {id} deleted");
        }
        Commands::Logout => {
            println!("logout ok");

            if Path::new(TOKEN_FILE).exists() {
                fs::remove_file(TOKEN_FILE)?;
                println!("token file removed {TOKEN_FILE}");
            } else {
                println!("token file is already absent");
            }
        }
        Commands::List { limit, offset } => {
            let page = client.list_posts(limit as i32, offset as i32).await?;
            println!(
                "total: {}, limit: {}, offset: {}",
                page.total, page.limit, page.offset
            );
            for post in page.posts {
                println!("#{} {}", post.id, post.title);
            }
        }
    }

    Ok(())
}

fn print_post(post: &blog_client::Post) {
    // чтобы было удобно грепать
    println!("id: {}", post.id);
    println!("title: {}", post.title);
    println!("content: {}", post.content);
    println!("author_id: {}", post.author_id);
    println!("created_at: {}", post.created_at);
    println!("updated_at: {}", post.updated_at);
}
