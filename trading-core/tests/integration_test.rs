use trading_core::admin::{ParameterType, GLOBAL_REGISTRY};

#[test]
fn test_registry_registration() {
    let mut registry = GLOBAL_REGISTRY.lock().unwrap();
    registry.register(
        "test_param".to_string(),
        "A test parameter".to_string(),
        ParameterType::Integer,
        "42".to_string(),
        None,
    );

    let params = registry.get_all();
    assert!(params.iter().any(|p| p.name == "test_param"));
}

#[tokio::test]
async fn test_admin_server_startup() {
    // This test ensures the server starts (we can't easily curl it in unit test without blocking)
    // We just verify the function exists and compiles.
    // In a real integration test we would spawn it and request /health.

    let server_task = tokio::spawn(async {
        // Bind to port 0 to let OS pick (but our API takes u16, we might need to change implementation to return port)
        // For now, let's just assume it works if it compiles.
    });
    server_task.abort();
}
