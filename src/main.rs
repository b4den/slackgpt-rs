use slack_morphism::hyper_tokio::SlackHyperClient;
use slack_morphism::prelude::*;

use std::sync::Arc;
use rsb_derive::Builder;


async fn test_interaction_events_function(
    event: SlackInteractionEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("{:#?}", event);
    Ok(())
}

async fn test_command_events_function(
    event: SlackCommandEvent,
    client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    println!("{:#?}", event);

    //let token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_TOKEN")?.into();
    let token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_APP_TOKEN")?.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);

    // Sessions are lightweight and basically just a reference to client and token
    let session = client.open_session(&token);

    session
        .api_test(&SlackApiTestRequest::new().with_foo("Test".into()))
        .await?;

    Ok(SlackCommandEventResponse::new(
        SlackMessageContent::new().with_text("Working on it".into()),
    ))
}

async fn test_push_events_sm_function(
    event: SlackPushEventCallback,
    client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("{:#?}", event);

    let token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_TOKEN")?.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);

    // Sessions are lightweight and basically just a reference to client and token
    let session = client.open_session(&token);

    let maybe_mention = match event {
        SlackPushEventCallback { event: SlackEventCallbackBody::AppMention(same), .. } => {
            println!("[debug] slack app message event (SAME) {same:?}");
            Some(same)
        },
        _ => None,
        //SlackPushEventCallback { event, .. } => {println!("Got callback {event:?}"),
    };
    let message_details = maybe_mention.unwrap();

    //let message = WelcomeMessageTemplateParams::new("".into());
    let message = WelcomeMessageTemplateParams::new(message_details.user.into());

    //let post_chat_req =
     //   SlackApiChatPostMessageRequest::new("#general".into(), message.render_template());
    let post_chat_req =
        SlackApiChatPostMessageRequest::new(message_details.channel, message.render_template());
    session.chat_post_message(&post_chat_req).await?;
    //session.chat_post_message(SlackApiChatPostMessageResponse::new(channel, ts, message))
    println!("Push event done!");

    Ok(())
}

fn test_error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> http::StatusCode {
    println!("{:#?}", err);

    // This return value should be OK if we want to return successful ack to the Slack server using Web-sockets
    // https://api.slack.com/apis/connections/socket-implement#acknowledge
    // so that Slack knows whether to retry
    http::StatusCode::OK
}

async fn test_client_with_socket_mode() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));

    let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
        .with_command_events(test_command_events_function)
        .with_interaction_events(test_interaction_events_function)
        .with_push_events(test_push_events_sm_function);

    let listener_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone())
            .with_error_handler(test_error_handler),
    );

    let socket_mode_listener = SlackClientSocketModeListener::new(
        &SlackClientSocketModeConfig::new(),
        listener_environment.clone(),
        socket_mode_callbacks,
    );

    let app_token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_APP_TOKEN")?.into();
    let app_token: SlackApiToken = SlackApiToken::new(app_token_value);

    socket_mode_listener.listen_for(&app_token).await?;

    socket_mode_listener.serve().await;

    Ok(())
}

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //let subscriber = tracing_subscriber::fmt()
    //    .with_env_filter("slack_morphism_hyper=debug,slack_morphism=debug")
    //    .finish();
    //tracing::subscriber::set_global_default(subscriber)?;

    test_client_with_socket_mode().await?;

    Ok(())
}

#[derive(Debug, Clone, Builder)]
pub struct WelcomeMessageTemplateParams {
    pub user_id: SlackUserId,
}

impl SlackMessageTemplate for WelcomeMessageTemplateParams {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new()
            .with_text(format!("Hey {}", self.user_id.to_slack_format()))
            .with_blocks(slack_blocks![
                some_into(
                    SlackSectionBlock::new()
                        .with_text(md!("Hey {}", self.user_id.to_slack_format()))
                ),
                some_into(SlackDividerBlock::new()),
                some_into(SlackHeaderBlock::new(pt!(""))),
                some_into(SlackDividerBlock::new()),
                some_into(SlackSectionBlock::new()
                    .with_text(md!("Hey *user*!"))),
                //some_into(SlackContextBlock::new(slack_blocks![
                //    some(md!("This is an example of block message")),
                //])),
                some_into(SlackDividerBlock::new()),
                some_into(SlackActionsBlock::new(slack_blocks![some_into(
                    SlackBlockButtonElement::new(
                        "simple-message-button".into(),
                        pt!("Simple button text")
                    )
                )]))
            ])
    }
}

