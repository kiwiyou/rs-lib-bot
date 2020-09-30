use std::env;

use tbot::prelude::*;

#[tokio::main]
async fn main() {
    let _guard = sentry::init(env::var("SENTRY_DSN").unwrap());

    let token = env::var("BOT_TOKEN").unwrap();
    let mut bot = tbot::Bot::new(token.clone()).event_loop();

    bot.text(|context| async move {
        let echo = &context.text.value;
        let call_result = context.send_message_in_reply(echo).call().await;

        if let Err(error) = call_result {
            sentry::capture_error(&error);
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
