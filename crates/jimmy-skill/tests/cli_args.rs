use jimmy_skill::cli::{get_prompt, resolve_system_prompt};

#[test]
fn test_at_file_reads_content() {
    let path = "/private/tmp/claude-502/test_sys_jimmy.txt";
    std::fs::write(path, "be helpful and concise").unwrap();

    let result = resolve_system_prompt(Some(&format!("@{path}"))).unwrap();
    assert_eq!(result, Some("be helpful and concise".to_string()));

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_at_file_missing_errors() {
    let result = resolve_system_prompt(Some("@/nonexistent/path/file.txt"));
    assert!(result.is_err(), "Should error on missing file");
}

#[test]
fn test_plain_string_system() {
    let result = resolve_system_prompt(Some("be nice")).unwrap();
    assert_eq!(result, Some("be nice".to_string()));
}

#[test]
fn test_none_system() {
    let result = resolve_system_prompt(None).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_get_prompt_with_arg() {
    let result = get_prompt(Some("hello".to_string()));
    assert_eq!(result, Ok("hello".to_string()));
}
