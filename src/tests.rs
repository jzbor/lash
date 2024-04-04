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
fn church_addition() {
    let mut interpreter = Interpreter::new();
    interpreter.interpret_std().unwrap();
    interpreter.set_church_num_enabled(true);
    // 5 + 2
    test_statement(&mut interpreter,
        "!normalize (ADD $5 $2)",
        "\\f . \\x . f (f (f (f (f (f (f x))))))");
    // 5 + 2
    test_statement(&mut interpreter,
        "!normalize (ADD $0 $0)",
        "\\f . \\x . x");
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
