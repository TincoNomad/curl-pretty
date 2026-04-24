use std::process::{Command, Stdio};
use std::io::Write;

fn create_temp_input(response: &str) -> std::process::Child {
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cat process");
    
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(response.as_bytes()).expect("Failed to write to stdin");
    }
    
    child
}

#[test]
fn test_integration_with_real_json_response() {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"id\": 1, \"name\": \"test\", \"active\": true}";
    let mut cat_child = create_temp_input(response);
    
    // Test curlp with the input
    let output = Command::new("cargo")
        .args(&["run", "--bin", "curlp", "--"])
        .stdin(cat_child.stdout.take().unwrap())
        .output()
        .expect("Failed to run curlp");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should contain the JSON structure
    assert!(stdout.contains("\"id\":"));
    assert!(stdout.contains("\"name\":"));
    assert!(stdout.contains("\"active\":"));
    assert!(stdout.contains("200 OK"));
}

#[test]
fn test_integration_with_xml_response() {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/xml\r\n\r\n<?xml version=\"1.0\"?><root><item>test</item></root>";
    let mut cat_child = create_temp_input(response);
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "curlp", "--"])
        .stdin(cat_child.stdout.take().unwrap())
        .output()
        .expect("Failed to run curlp");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should contain XML structure
    assert!(stdout.contains("<?xml"));
    assert!(stdout.contains("<root>"));
    assert!(stdout.contains("<item>"));
}

#[test]
fn test_integration_with_plain_text_response() {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!\nThis is plain text.";
    let mut cat_child = create_temp_input(response);
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "curlp", "--"])
        .stdin(cat_child.stdout.take().unwrap())
        .output()
        .expect("Failed to run curlp");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should contain the plain text
    assert!(stdout.contains("Hello, World!"));
    assert!(stdout.contains("This is plain text"));
}

#[test]
fn test_integration_error_response() {
    let response = "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{\"error\": \"Not found\", \"code\": 404}";
    let mut cat_child = create_temp_input(response);
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "curlp", "--"])
        .stdin(cat_child.stdout.take().unwrap())
        .output()
        .expect("Failed to run curlp");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should contain error information
    assert!(stdout.contains("404 Not Found"));
    assert!(stdout.contains("\"error\":"));
    assert!(stdout.contains("\"code\":"));
}

#[test]
fn test_integration_empty_response() {
    let response = "HTTP/1.1 204 No Content\r\n\r\n";
    let mut cat_child = create_temp_input(response);
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "curlp", "--"])
        .stdin(cat_child.stdout.take().unwrap())
        .output()
        .expect("Failed to run curlp");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should handle empty body gracefully
    assert!(stdout.contains("204 No Content"));
    assert!(stdout.contains("(respuesta sin cuerpo)"));
}

#[test]
fn test_integration_redirect_response() {
    let response = "HTTP/1.1 301 Moved Permanently\r\nLocation: /new-url\r\n\r\nHTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"success\": true, \"redirected\": true}";
    let mut cat_child = create_temp_input(response);
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "curlp", "--"])
        .stdin(cat_child.stdout.take().unwrap())
        .output()
        .expect("Failed to run curlp");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should handle redirects and show final response
    assert!(stdout.contains("200 OK"));
    assert!(stdout.contains("\"success\":"));
    assert!(stdout.contains("\"redirected\":"));
}
