#![cfg(all(feature = "desktop", windows))]

use aido_core::ai_client::{ChatMessage, ChatRole};
use aido_core::companion_runtime::{
    check_model_profile, suggest_wsl_model_profile, CompanionRuntime,
};

#[test]
#[ignore = "requires the reference PC's CUDA-capable Ubuntu runtime and local Qwen GGUF"]
fn wsl_companion_answers_with_session_context() {
    let profile = suggest_wsl_model_profile().expect("WSL model discovery");
    assert!(check_model_profile(&profile).unwrap().contains("CUDA"));
    let runtime = CompanionRuntime::default();
    let mut conversation = vec![ChatMessage {
        role: ChatRole::User,
        content: "Was ist 17 mal 23? Antworte nur mit der Zahl.".into(),
    }];
    let first = runtime
        .chat_with_profile(&conversation, &profile)
        .expect("first WSL model answer");
    assert!(first.contains("391"), "{first}");
    conversation.push(ChatMessage {
        role: ChatRole::Assistant,
        content: first,
    });
    conversation.push(ChatMessage {
        role: ChatRole::User,
        content: "Addiere eins zu deiner letzten Antwort. Antworte nur mit der Zahl.".into(),
    });
    let second = runtime
        .chat_with_profile(&conversation, &profile)
        .expect("follow-up WSL model answer");
    assert!(second.contains("392"), "{second}");
    runtime.shutdown();
}
