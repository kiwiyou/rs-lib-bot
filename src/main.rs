use std::{env, sync::Arc};

use log::*;
use reqwest::{Client, ClientBuilder};
use tbot::contexts;
use tbot::types as telegram;

mod search;
mod util;

struct State {
    client: Client,
}

#[tokio::main]
async fn main() {
    let _guard = sentry::init(env::var("SENTRY_DSN").unwrap());
    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN").unwrap();
    let state = State {
        client: ClientBuilder::new()
            .user_agent("rs-lib-bot (kiwiyou.dev@gmail.com)")
            .build()
            .expect("Failed to create request client"),
    };
    let mut bot = tbot::Bot::new(token.clone()).stateful_event_loop(state);

    bot.inline(|context, state| async move {
        if let Err(error) = handle_inline_query(context, state).await {
            sentry_anyhow::capture_anyhow(&error);
        }
    });

    let webhook_url = env::var("WEBHOOK_URL").unwrap();
    let port = env::var("PORT").unwrap().parse().unwrap();
    bot.webhook(&format!("{}/{}", webhook_url, &token), port)
        .accept_updates_on(format!("/{}", &token))
        .ip("0.0.0.0".parse().unwrap())
        .http()
        .start()
        .await
        .unwrap();
}

async fn handle_inline_query(
    context: Arc<contexts::Inline>,
    state: Arc<State>,
) -> anyhow::Result<()> {
    use telegram::inline_query;
    let query = &context.query;

    trace!("Inline Query: {}", query);
    let client = &state.client;
    if search::crate_exists(client, query).await? {
        info!("Valid Crate Query: {}", query);

        let info = search::get_crate_info(client, query).await?;
        let text = util::TextBuilder::new()
            .text("", &info.name, "")
            .text(" ", &info.newest_version, "")
            .text_opt(" ", &info.license, " License")
            .text_opt("\n", &info.description, "\n")
            .text(
                "\n",
                info.recent_downloads.to_string(),
                " download(s) recently",
            )
            .text(" (", info.downloads.to_string(), " total)")
            .build();

        let content = telegram::input_message_content::Text::new(&text);
        let result =
            inline_query::Result::new(query, inline_query::result::Article::new(query, content));
        context.answer(&[result]).call().await?;
    }

    Ok(())
}
