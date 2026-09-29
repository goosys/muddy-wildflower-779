use loco_rs::testing::prelude::*;
use myapp::{app::App, views::haikunator::GeneratorResponse};

fn assert_generated_name(name: &str) {
    let parts: Vec<_> = name.split('-').collect();
    assert_eq!(parts.len(), 3);
    assert!(!parts[0].is_empty());
    assert!(!parts[1].is_empty());
    assert_eq!(parts[2].len(), 4);
    assert!(parts[2].chars().all(|character| character.is_ascii_digit()));
}

#[tokio::test]
async fn generate_endpoints_return_memorable_names() {
    request::<App, _, _>(|request, _context| async move {
        let json_response = request.get("/api/gen").await;
        json_response.assert_status_ok();
        let generated: GeneratorResponse = json_response.json();
        assert_generated_name(&generated.name);

        let text_response = request.get("/api/gen.txt").await;
        text_response.assert_status_ok();
        assert_generated_name(&text_response.text());
    })
    .await;
}
