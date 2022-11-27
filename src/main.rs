use std::{env, future::Future, io, net::SocketAddr};

use axum::Router;
use futures_util::FutureExt;
use teloxide::{
    adaptors::CacheMe,
    dispatching::{update_listeners::webhooks, UpdateHandler},
    prelude::*,
    types::MessageKind,
    RequestError,
};
use tokio::try_join;
use tracing::info;
use url::Url;

#[cfg(feature = "tls")]
const DEFAULT_PORT: u16 = 443;

#[cfg(not(feature = "tls"))]
const DEFAULT_PORT: u16 = 80;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting bot");
    let bot = Bot::from_env().cache_me();
    info!("Bot started");
    let mut dispatcher = Dispatcher::builder(bot.clone(), create_update_handler())
        .enable_ctrlc_handler()
        .build();

    if env::var("USE_POLLING").is_ok() {
        dispatcher.dispatch().await;
    } else {
        use std::fmt::Write;

        let port = env::var("PORT")
            .map(|raw_port| raw_port.parse::<u16>().expect("PORT value to be integer"))
            .ok();
        let host = env::var("HOST").expect("have HOST env variable");
        let teloxide_token = bot.inner().token();
        let mut url = format!("https://{host}");
        write!(url, "/bot{teloxide_token}").unwrap();
        let url = Url::parse(&url).unwrap();

        let webhooks_options =
            webhooks::Options::new(([0, 0, 0, 0], port.unwrap_or(DEFAULT_PORT)).into(), url);
        let (listener, stop_fut, router) = webhooks::axum_to_router(bot.clone(), webhooks_options)
            .await
            .expect("Couldn't create axum router");

        let axum_future = setup_server(router, port, stop_fut);

        try_join!(
            axum_future,
            dispatcher
                .dispatch_with_listener(listener, LoggingErrorHandler::new())
                .map(Ok),
        )
        .unwrap();
    }
}

#[cfg(feature = "tls")]
async fn setup_server(
    app: Router,
    port: Option<u16>,
    stop_fut: impl Future<Output = ()>,
) -> Result<(), io::Error> {
    use axum_server::tls_rustls::RustlsConfig;
    use std::path::PathBuf;

    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env::var("SSL_CERT").expect("cannot read SSL_CERT env var")),
        PathBuf::from(env::var("SSL_KEY").expect("cannot read SSL_CERT env var")),
    )
    .await
    .expect("unable to create TLS config from cert and key file");

    let addr = SocketAddr::from(([0, 0, 0, 0], port.unwrap_or(DEFAULT_PORT)));
    let handle = axum_server::Handle::new();
    try_join!(
        axum_server::bind_rustls(addr, config)
            .handle(handle.clone())
            .serve(app.into_make_service()),
        async move {
            stop_fut.await;
            handle.graceful_shutdown(None);
            Ok(())
        }
    )?;

    Ok(())
}

#[cfg(not(feature = "tls"))]
async fn setup_server(
    app: Router,
    port: Option<u16>,
    stop_fut: impl Future<Output = ()>,
) -> Result<(), io::Error> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port.unwrap_or(DEFAULT_PORT)));
    let handle = axum_server::Handle::new();
    try_join!(
        axum_server::bind(addr)
            .handle(handle.clone())
            .serve(app.into_make_service()),
        async move {
            stop_fut.await;
            handle.graceful_shutdown(None);
            Ok(())
        }
    )?;

    Ok(())
}

fn create_update_handler() -> UpdateHandler<RequestError> {
    Update::filter_message().endpoint(handle_messages)
}

async fn handle_messages(bot: CacheMe<Bot>, message: Message) -> Result<(), RequestError> {
    use std::fmt::Write;

    if let MessageKind::NewChatMembers(members) = &message.kind {
        for member in &members.new_chat_members {
            let mut text = "Ciao ".to_string();
            match &member.username {
                Some(username) => write!(text, "@{}", username).unwrap(),
                None => text.push_str(&member.first_name),
            }
            text.push_str(
                " e benvenuto/a nel gruppo italiano dedicato a Rust: linguaggio di programmazione \
                 di sistema a elevate prestazioni che previene errori di segmentazione e \
                 garantisce la sicurezza dei dati tra i thread.\n\n\
                 In questo gruppo, potrai parlare di tutto l'ecosistema Rust e chiedere supporto o \
                 consigli!\n\n\
                 E ricorda: non è il gruppo del gioco™\n\
                 Buona permanenza 😉",
            );
            bot.send_message(message.chat.id, text).await?;
        }
    }
    Ok(())
}
