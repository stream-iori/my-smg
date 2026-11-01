use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Sse, sse::Event},
    routing::{get, post},
};
use futures_util::{Stream, stream};
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    env, process,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    worker_id: String,
    delay: Duration,
    failures_remaining: Arc<Mutex<usize>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    worker_id: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    worker_id: String,
    reply: String,
}

//State(state) 这是模式匹配,取出内部appState
async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        worker_id: state.worker_id,
    })
}

async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> (StatusCode, Json<ChatResponse>) {
    let should_fail = {
        let mut remaining = match state.failures_remaining.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if *remaining == 0 {
            false
        } else {
            *remaining -= 1;
            true
        }
    };

    tokio::time::sleep(state.delay).await;

    let (status, reply) = if should_fail {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "simulated failure".to_string(),
        )
    } else {
        (StatusCode::OK, format!("echo: {}", request.message))
    };

    (
        status,
        Json(ChatResponse {
            worker_id: state.worker_id,
            reply,
        }),
    )
}

async fn chat_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = stream::unfold((1_usize, state.delay), |(part, delay)| async move {
        if part > 3 {
            return None;
        }

        if part > 1 {
            tokio::time::sleep(delay).await;
        }

        let event = Event::default().data(format!("part-{part}"));

        Some((
            //交给流的使用者
            Ok::<Event, Infallible>(event),
            //留给下一次生成
            (part + 1, delay),
        ))
    });

    //它把流包装成 SSE 响应，并转移流的所有权
    Sse::new(events)
}

#[tokio::main] //create Runtime and execute async main
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const USAGE: &str =
        "usage: fake-worker <worker-id> <listen-address> <delay-ms> <failure-count>";

    let mut args = env::args().skip(1);

    let Some(worker_id) = args.next() else {
        eprintln!("{USAGE}");
        process::exit(2);
    };

    let Some(listen_address) = args.next() else {
        eprintln!("{USAGE}");
        process::exit(2);
    };

    let delay_ms = match args.next() {
        Some(delay_text) => match delay_text.parse::<u64>() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("invalid delay-ms {delay_text:?}: {error}");
                process::exit(2);
            }
        },
        None => {
            eprintln!("usage: fake-worker <worker-id> <listen-address> <delay-ms>");
            process::exit(2);
        }
    };

    let failure_count = match args.next() {
        Some(text) => match text.parse::<usize>() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("invalid failure_count {text:?}: {error}");
                process::exit(2);
            }
        },
        None => {
            eprint!("{USAGE}");
            process::exit(2);
        }
    };

    if args.next().is_some() {
        eprintln!("usage fake-worker <worker-id> <listen-address>");
        process::exit(2);
    }

    let listener = TcpListener::bind(listen_address.as_str()).await?;

    println!("fake worker {worker_id} listening on http://{listen_address}");

    let state = AppState {
        worker_id,
        delay: Duration::from_millis(delay_ms),
        failures_remaining: Arc::new(Mutex::new(failure_count)),
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .route("/chat/stream", post(chat_stream))
        .with_state(state);

    //A Serve Type be returned which implementation all the function of future
    //async create a future, and await meaning wait future complete
    axum::serve(listener, app).await?;

    Ok(())
}
