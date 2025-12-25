mod common;

#[test]
fn test_int_literal() {
    assert_int_result!(
        r#"
        int main() {
            return 42;
        }
    "#,
        42
    );
}

#[test]
fn test_int_addition() {
    assert_int_result!(
        r#"
        int main() {
            int a = 10;
            int b = 20;
            return a + b;
        }
    "#,
        30
    );
}

#[test]
fn test_int_subtraction() {
    assert_int_result!(
        r#"
        int main() {
            int a = 50;
            int b = 20;
            return a - b;
        }
    "#,
        30
    );
}

#[test]
fn test_int_multiplication() {
    assert_int_result!(
        r#"
        int main() {
            int a = 6;
            int b = 7;
            return a * b;
        }
    "#,
        42
    );
}

#[test]
fn test_int_division() {
    assert_int_result!(
        r#"
        int main() {
            int a = 84;
            int b = 2;
            return a / b;
        }
    "#,
        42
    );
}

#[test]
fn test_int_complex_expression() {
    assert_int_result!(
        r#"
        int main() {
            int a = 5;
            int b = 3;
            int c = 2;
            return (a + b) * c - 4;
        }
    "#,
        12
    ); // (5 + 3) * 2 - 4 = 16 - 4 = 12
}

#[test]
fn test_int_negative() {
    assert_int_result!(
        r#"
        int main() {
            int a = 10;
            return -a;
        }
    "#,
        -10
    );
}

#[test]
fn test_int_assignment_chain() {
    assert_int_result!(
        r#"
        int main() {
            int a = 5;
            int b = a + 10;
            int c = b * 2;
            return c;
        }
    "#,
        30
    ); // (5 + 10) * 2 = 30
}

#[test]
fn test_do_while_loop() {
    assert_int_result!(
        r#"
        int main() {
            int i = 0;
            int sum = 0;
            do {
                sum = sum + i;
                i = i + 1;
            } while (i < 5);
            return sum;
        }
    "#,
        10
    ); // sum of 0+1+2+3+4 = 10
}

