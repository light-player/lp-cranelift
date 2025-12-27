mod common;

#[test]
fn test_bool_true() {
    assert_bool_result!(
        r#"
        bool main() {
            return true;
        }
    "#,
        true
    );
}

#[test]
fn test_bool_false() {
    assert_bool_result!(
        r#"
        bool main() {
            return false;
        }
    "#,
        false
    );
}

#[test]
fn test_bool_not() {
    assert_bool_result!(
        r#"
        bool main() {
            bool t = true;
            return !t;
        }
    "#,
        false
    );
}

#[test]
fn test_int_comparison_eq() {
    assert_bool_result!(
        r#"
        bool main() {
            int a = 42;
            int b = 42;
            return a == b;
        }
    "#,
        true
    );
}

#[test]
fn test_int_comparison_ne() {
    assert_bool_result!(
        r#"
        bool main() {
            int a = 10;
            int b = 20;
            return a != b;
        }
    "#,
        true
    );
}

#[test]
fn test_int_comparison_lt() {
    assert_bool_result!(
        r#"
        bool main() {
            int a = 10;
            int b = 20;
            return a < b;
        }
    "#,
        true
    );
}

#[test]
fn test_int_comparison_gt() {
    assert_bool_result!(
        r#"
        bool main() {
            int a = 30;
            int b = 20;
            return a > b;
        }
    "#,
        true
    );
}

#[test]
fn test_int_comparison_complex() {
    assert_bool_result!(
        r#"
        bool main() {
            int a = 5;
            int b = 10;
            int c = 15;
            return (a + b) == c;
        }
    "#,
        true
    );
}
