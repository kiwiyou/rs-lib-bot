use std::{env, sync::Arc};

use anyhow::Context;
use log::*;
use reqwest::{Client, ClientBuilder};
use tbot::contexts;
use tbot::types as telegram;

mod search;
mod util;

struct State {
    client: Client,
    search_url: String,
}

#[cfg(feature = "error-report")]
struct ErrorGuard {
    _guard: sentry::ClientInitGuard,
}

#[cfg(feature = "error-report")]
impl ErrorGuard {
    fn init() -> Self {
        Self {
            _guard: sentry::init(env::var("SENTRY_DSN").unwrap()),
        }
    }
}

#[cfg(not(feature = "error-report"))]
struct ErrorGuard;

#[cfg(not(feature = "error-report"))]
impl ErrorGuard {
    fn init() -> Self {
        Self
    }
}

#[tokio::main]
async fn main() {
    let _guard = ErrorGuard::init();
    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN").unwrap();
    let state = State {
        client: ClientBuilder::new()
            .user_agent("rs-lib-bot (kiwiyou.dev@gmail.com)")
            .build()
            .expect("Failed to create request client"),
        search_url: env::var("SEARCH_URL").unwrap(),
    };
    let mut bot = tbot::Bot::new(token.clone()).stateful_event_loop(state);

    bot.inline(|context, state| async move {
        if let Err(error) = handle_inline_query(context.clone(), state)
            .await
            .with_context(|| format!("Error in inline query: {:#?}", *context))
        {
            #[cfg(feature = "error-report")]
            sentry_anyhow::capture_anyhow(&error);
            error!("{}", error);
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
    if query.is_empty() {
        return Ok(());
    }

    debug!("Inline Query: {}", query);
    let client = &state.client;
    if let Some(info) = search::search_crate(client, &state.search_url, query).await? {
        let description = info.description.map(|desc| desc.replace('\n', " "));
        let crate_size = info
            .crate_size
            .map(|size| size.file_size(file_size_opts::BINARY).unwrap());
        let deps = {
            let mut buffer = String::new();
            if info.dev_dependencies > 0 {
                buffer.push_str(&info.dev_dependencies.to_string());
                buffer.push_str(" for dev");
            }
            if info.build_dependencies > 0 {
                if !buffer.is_empty() {
                    buffer.push_str(", ");
                }
                buffer.push_str(&info.build_dependencies.to_string());
                buffer.push_str(" for build");
            }
            if buffer.is_empty() {
                None
            } else {
                Some(buffer)
            }
        };
        let dependency_label = if info.dependencies == 1 {
            " Dependency"
        } else {
            " Dependencies"
        };
        let text = util::TextBuilder::new()
            .text("📦 *", &escape_markdown(query), "*")
            .text(" _", &escape_markdown(&info.version), "_")
            .text_opt(
                ", ",
                &info.license.as_deref().map(escape_markdown),
                " License",
            )
            .text_opt(" \\(", &crate_size.as_deref().map(escape_markdown), "\\)")
            .text_opt(
                "\n\n",
                &description.as_deref().map(str::trim).map(escape_markdown),
                "\n",
            )
            .text(
                "\n📥 All\\-Time ",
                info.downloads.to_formatted_string(&Locale::en),
                "",
            )
            .text(
                "\n🕒 Last Update ",
                HumanTime::from(info.updated_at).to_string(),
                "",
            )
            .text("\n📊 ", info.dependencies.to_string(), dependency_label)
            .text_opt(" \\(", &deps, "\\)")
            .build();

        let mut buttons = Vec::new();
        let docs = info
            .documentation
            .unwrap_or(format!("https://docs.rs/{}", query));
        {
            if let Some(homepage) = &info.homepage {
                buttons.push(keyboard::Button::new(
                    "🏠 Home",
                    keyboard::ButtonKind::Url(&homepage),
                ));
            }
            buttons.push(keyboard::Button::new(
                "📚 Docs",
                keyboard::ButtonKind::Url(&docs),
            ));
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
        let results = vec![result];
        let request = context.answer(&results);
        request
            .clone()
            .call()
            .await
            .with_context(move || format!("Error in request: {:#?}", request))?;
    }

    Ok(())
}
