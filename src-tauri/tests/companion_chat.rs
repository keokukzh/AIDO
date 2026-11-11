use aido_core::ai_client::{ChatMessage, ChatRole, LocalAiClient};
use aido_core::permission_gateway::{AccessMode, PermissionGateway, PrivacyMode};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

#[test]
fn companion_chat_sends_history_to_the_local_model() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let client = LocalAiClient::with_endpoints(
        PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline),
        format!("http://{}", listener.local_addr().unwrap()),
        "http://127.0.0.1:18080",
        "test-qwen",
    );
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim_end(), "POST /v1/chat/completions HTTP/1.1");
        let mut length = 0;
        let mut authorized = false;
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if line.trim().is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').unwrap();
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().unwrap();
            }
            if name.eq_ignore_ascii_case("authorization") {
                authorized = value.trim() == "Bearer test-key";
            }
        }
        assert!(authorized);
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["model"], "test-qwen");
        assert_eq!(payload["messages"][0]["role"], "system");
        assert_eq!(payload["messages"][1]["content"], "17 mal 23?");
        assert_eq!(payload["messages"][2]["role"], "assistant");
        assert_eq!(payload["messages"][3]["content"], "Plus eins?");
        let response = r#"{"choices":[{"message":{"content":"392"}}]}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
            response.len()
        )
        .unwrap();
    });
    let messages = [
        ChatMessage {
            role: ChatRole::User,
            content: "17 mal 23?".into(),
        },
        ChatMessage {
            role: ChatRole::Assistant,
            content: "391".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Plus eins?".into(),
        },
    ];
    assert_eq!(
        client.companion_chat(&messages, Some("test-key")).unwrap(),
        "392"
    );
    server.join().unwrap();
}
