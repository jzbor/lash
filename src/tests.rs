use crate::interpreter::Interpreter;


#[allow(dead_code)]
fn test_statement(interpreter: &mut Interpreter, input: &str, expected: &str) {
    let statement = interpreter.interpret_line(input).unwrap();
    let serialized = format!("{}", statement);
    assert_eq!(serialized, expected);

}

#[test]
fn id() {
    let mut interpreter = Interpreter::new();
    test_statement(&mut interpreter, "!normalize ((\\x . x) x)", "x");
}

#[test]
fn issue_1_capture_avoidance() {
    let mut interpreter = Interpreter::new();
    // the full term
    test_statement(&mut interpreter,
        "!normalize ((\\f x. f (f x)) (\\f x. f (f x)))",
        "\\x . \\x' . x (x (x (x x')))");
    // the critical part
    test_statement(&mut interpreter,
        "!reduce (\\x . (\\f . \\x . f (f x)) (\\x' . x (x x')))",
        "\\x . \\x' . (\\x' . x (x x')) ((\\x' . x (x x')) x')");

}
