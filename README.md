# Slack GPT App

This app uses the `chatgpt-rs` crate for interacting with ChatGPT.

You need to have `SLACK_TEST_TOKEN` and `SLACK_TEST_APP_TOKEN` present.

`SLACK_TEST_TOKEN` exists under oAuth and permissions, specifically the bot user token.

`SLACK_TEST_APP_TOKEN` exists under `Basic Information` -> `App Level Tokens` with the scope of connections:write.
