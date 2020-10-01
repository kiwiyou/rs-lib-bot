use std::{env, sync::Arc, time::Duration};

use log::*;
use reqwest::{Client, ClientBuilder};
use tbot::contexts;
use tbot::types as telegram;
use tokio::{sync::Mutex, time::Instant};

mod search;
mod util;

struct State {
    client: Client,
    no_crate_req_until: Mutex<Instant>,
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
        no_crate_req_until: Mutex::new(Instant::now()),
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
    use chrono_humanize::HumanTime;
    use humansize::{file_size_opts, FileSize};
    use num_format::{Locale, ToFormattedString};
    use telegram::inline_query;
    use telegram::keyboard::inline as keyboard;
    use util::escape_markdown;
    let query = &context.query;

    debug!("Inline Query: {}", query);
    let client = &state.client;
    if search::crate_exists(client, query).await? {
        info!("Valid Crate Query: {}", query);

        let mut no_crate_req_until = state.no_crate_req_until.lock().await;
        tokio::time::delay_until(*no_crate_req_until).await;
        debug!("Sent crates.io request for crate `{}`", query);
        let info = search::get_crate_info(client, query).await?;
        *no_crate_req_until = Instant::now().checked_add(Duration::from_secs(1)).unwrap();
        drop(no_crate_req_until);

        let crate_size = info
            .crate_size
            .map(|size| size.file_size(file_size_opts::BINARY).unwrap());
        let description = info.description.map(|s| escape_markdown(s.trim()));
        let text = util::TextBuilder::new()
            .text("📦 *", &escape_markdown(&info.name), "*")
            .text(" _", &info.newest_version, "_")
            .text_opt(", ", &info.license, " License")
            .text_opt(" (", &crate_size, ")")
            .text_opt("\n\n", &description, "\n")
            .text(
                "\n📥 All-Time ",
                info.downloads.to_formatted_string(&Locale::en),
                "",
            )
            .text(
                "\n📥 Recent ",
                info.recent_downloads.to_formatted_string(&Locale::en),
                "",
            )
            .text(
                "🕒 Last Update ",
                HumanTime::from(info.updated_at).to_string(),
                "",
            )
            .build();

        let mut buttons = Vec::new();
        {
            if let Some(homepage) = &info.homepage {
                buttons.push(keyboard::Button::new(
                    "🏠 Home",
                    keyboard::ButtonKind::Url(&homepage),
                ));
            }
            if let Some(docs) = &info.documentation {
                buttons.push(keyboard::Button::new(
                    "📚 Docs",
                    keyboard::ButtonKind::Url(&docs),
                ));
            }
            if let Some(repo) = &info.repository {
                buttons.push(keyboard::Button::new(
                    "📂 Repo",
                    keyboard::ButtonKind::Url(&repo),
                ));
            }
        }
        let mut container = Vec::new();
        container.push(buttons.as_slice());
        let keyboard = keyboard::Keyboard::new(&container);

        let content = telegram::input_message_content::Text::new(
            telegram::parameters::Text::markdown_v2(&text),
        );

        let result =
            inline_query::Result::new(query, inline_query::result::Article::new(query, content))
                .reply_markup(keyboard);
        context.answer(&[result]).call().await?;
    }

    Ok(())
}
