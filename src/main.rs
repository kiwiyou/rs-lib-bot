use std::{env, sync::Arc};

use log::*;
use reqwest::{Client, ClientBuilder};
use tbot::contexts;
use tbot::types as telegram;

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
    if crate_exists(client, query).await? {
        info!("Valid Crate Query: {}", query);
        let content = telegram::input_message_content::Text::new("It exists!");
        let result = inline_query::Result::new(
            "Your crate",
            inline_query::result::Article::new(query, content),
        );
        context.answer(&[result]).call().await?;
    }

    Ok(())
}

async fn crate_exists(client: &Client, name: &str) -> anyhow::Result<bool> {
    let name = name.to_ascii_lowercase();

    let url = if name.len() <= 2 {
        format!(
            "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/{}/{}",
            name.len(),
            name
        )
    } else if name.len() == 3 {
        format!(
            "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/3/{}/{}",
            &name[..1],
            name
        )
    } else {
        format!(
            "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/{}/{}/{}",
            &name[..2],
            &name[2..4],
            name
        )
    };

    let response = client.head(&url).send().await?;
    Ok(response.status().is_success())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn crate_exists_works() {
        let client = Client::new();
        let crates_and_existence = [
            ("a", true),
            ("at", true),
            ("top", true),
            ("surf", true),
            ("tokio", true),
            ("google-gamesconfiguration1_configuration-cli", true),
            ("_", false),
            ("a_", false),
            ("b!g", false),
            ("g0od", false),
            ("q_e_d", false),
            (
                "this_crate_has_so_long_name_that_it_exceeds_64_letters_and_blocked_by_crates_io",
                false,
            ),
        ];
        for &(name, existence) in &crates_and_existence {
            let name_upper = name.to_ascii_uppercase();

            let result = crate_exists(&client, &name).await.ok();
            let result_upper = crate_exists(&client, &name_upper).await.ok();

            assert_eq!(Some(existence), result);
            assert_eq!(Some(existence), result_upper);
        }
    }
}
